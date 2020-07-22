use hecs::World;
use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::SystemClosure;
use crate::{ArchetypeSet, BorrowSet, ExecutorBuilder, ResourceTuple, SystemId};

mod dispatcher;
mod scheduler;
mod scheduler_disjoint;

use dispatcher::Dispatcher;
use scheduler::Scheduler;
use scheduler_disjoint::DisjointScheduler;

static DISCONNECTED: &str = "channel should not be disconnected at this point";
static INVALID_ID: &str = "system IDs should always be valid";

/// Container for systems and their condensed metadata.
struct System<'closure, Resources>
where
    Resources: ResourceTuple,
{
    closure: Arc<Mutex<SystemClosure<'closure, Resources::Wrapped>>>,
    resource_set: BorrowSet,
    component_set: BorrowSet,
    archetype_writer: Box<dyn Fn(&World, &mut ArchetypeSet) + Send>,
    dependants: Vec<SystemId>,
    dependencies: usize,
}

/// Typed `usize` used to cache the amount of dependants the system associated
/// with a `SystemId` has; avoids hashmap lookups while sorting.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct DependantsLength(pub usize);

/// Variants of parallel executor, chosen based on properties of systems in the builder.
pub enum ExecutorParallel<'closures, Resources>
where
    Resources: ResourceTuple,
{
    // TODO consider more granularity:
    // scheduler, disjoint scheduler, dispatcher (has to be disjoint either way)
    /// Used when all systems are proven to be statically disjoint
    /// and have no dependencies.
    Dispatching(Dispatcher<'closures, Resources>),
    /// Used when systems cannot be proven to be statically disjoint,
    /// or have dependencies.
    Scheduling(Scheduler<'closures, Resources>),
    DisjointScheduling(DisjointScheduler<'closures, Resources>),
}

impl<'closures, Resources> ExecutorParallel<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        // This will cache dependencies for later conversion into dependants.
        let mut all_dependencies = Vec::new();
        let mut systems_without_dependencies = Vec::new();
        let ExecutorBuilder {
            mut systems,
            mut all_component_types,
            ..
        } = builder;
        // This guarantees iteration order; TODO probably not necessary?..
        let all_component_types = all_component_types.drain().collect::<Vec<_>>();
        // Convert systems from builder representation to executor representation,
        // splitting off dependency collections and condensing component type sets into bitsets.
        let mut systems: HashMap<SystemId, System<'closures, Resources>> = systems
            .drain()
            .map(|(id, system)| {
                let dependencies = system.dependencies.len();
                // Remember systems with no dependencies, these will be queued first on run.
                if dependencies == 0 {
                    systems_without_dependencies.push(id);
                }
                all_dependencies.push((id, system.dependencies));
                (
                    id,
                    System {
                        closure: Arc::new(Mutex::new(system.closure)),
                        resource_set: system.resource_set,
                        component_set: system.component_type_set.condense(&all_component_types),
                        archetype_writer: system.archetype_writer,
                        dependants: vec![],
                        dependencies,
                    },
                )
            })
            .collect();
        // Check if all systems can be ran concurrently, ignoring dependencies.
        let all_disjoint = {
            let mut all_disjoint = true;
            let mut tested_ids = Vec::new();
            'outer: for (id, system) in &systems {
                tested_ids.push(*id);
                for (id, other) in &systems {
                    if !tested_ids.contains(id)
                        && (!system.resource_set.is_compatible(&other.resource_set)
                            || !system.component_set.is_compatible(&other.component_set))
                    {
                        all_disjoint = false;
                        break 'outer;
                    }
                }
            }
            all_disjoint
        };
        // If all systems are disjoint and independent, dispatching heuristic may be used.
        if all_disjoint && systems.len() == systems_without_dependencies.len() {
            return ExecutorParallel::Dispatching(Dispatcher {
                systems: systems
                    .drain()
                    .map(|(id, system)| (id, system.closure))
                    .collect(),
            });
        }
        // Convert system-dependencies mapping to system-dependants mapping.
        for (dependant_id, mut dependencies) in all_dependencies.drain(..) {
            for dependee_id in dependencies.drain(..) {
                systems
                    .get_mut(&dependee_id)
                    .expect(INVALID_ID)
                    .dependants
                    .push(dependant_id);
            }
        }
        // Cache amount of dependants the system has.
        let mut systems_without_dependencies: Vec<_> = systems_without_dependencies
            .drain(..)
            .map(|id| {
                (
                    id,
                    DependantsLength(systems.get(&id).expect(INVALID_ID).dependants.len()),
                )
            })
            .collect();
        // Sort independent systems so that those with most dependants are queued first.
        systems_without_dependencies.sort_by(|(_, a), (_, b)| b.cmp(a));
        // This should be guaranteed by the builder's logic anyway.
        debug_assert!(!systems_without_dependencies.is_empty());
        let (sender, receiver) = crossbeam_channel::unbounded();
        if all_disjoint {
            ExecutorParallel::DisjointScheduling(DisjointScheduler {
                systems: systems
                    .drain()
                    .map(|(id, system)| {
                        (
                            id,
                            scheduler_disjoint::System {
                                closure: system.closure,
                                dependants: system.dependants,
                                dependencies: system.dependencies,
                                unsatisfied_dependencies: 0,
                            },
                        )
                    })
                    .collect(),
                archetypes_generation: None,
                systems_without_dependencies,
                systems_to_run_now: Vec::new(),
                systems_running: 0,
                systems_just_finished: Vec::new(),
                systems_to_decrement_dependencies: Vec::new(),
                sender,
                receiver,
            })
        } else {
            ExecutorParallel::Scheduling(Scheduler {
                systems: systems
                    .drain()
                    .map(|(id, system)| {
                        (
                            id,
                            scheduler::System {
                                closure: system.closure,
                                resource_set: system.resource_set,
                                component_set: system.component_set,
                                archetype_set: Default::default(),
                                archetype_writer: system.archetype_writer,
                                dependants: system.dependants,
                                dependencies: system.dependencies,
                                unsatisfied_dependencies: 0,
                            },
                        )
                    })
                    .collect(),
                archetypes_generation: None,
                systems_without_dependencies,
                systems_to_run_now: Vec::new(),
                systems_running: HashSet::new(),
                systems_just_finished: Vec::new(),
                systems_to_decrement_dependencies: Vec::new(),
                sender,
                receiver,
            })
        }
    }

    pub fn force_archetype_recalculation(&mut self) {
        use ExecutorParallel::*;
        match self {
            Dispatching(_) => (),
            Scheduling(scheduler) => scheduler.archetypes_generation = None,
            DisjointScheduling(_) => (),
        }
    }

    pub fn run(&mut self, world: &World, wrapped: Resources::Wrapped) {
        use ExecutorParallel::*;
        match self {
            Dispatching(dispatcher) => dispatcher.run(world, wrapped),
            Scheduling(scheduler) => scheduler.run(world, wrapped),
            DisjointScheduling(disjoint_scheduler) => disjoint_scheduler.run(world, wrapped),
        }
    }

    #[cfg(test)]
    fn unwrap_to_dispatcher(self) -> Dispatcher<'closures, Resources> {
        use ExecutorParallel::*;
        match self {
            Dispatching(dispatcher) => dispatcher,
            Scheduling(_) => panic!("produced executor is a scheduler"),
            DisjointScheduling(_) => panic!("produced executor is a disjoint scheduler"),
        }
    }

    #[cfg(test)]
    fn unwrap_to_scheduler(self) -> Scheduler<'closures, Resources> {
        use ExecutorParallel::*;
        match self {
            Dispatching(_) => panic!("produced executor is a dispatcher"),
            Scheduling(scheduler) => scheduler,
            DisjointScheduling(_) => panic!("produced executor is a disjoint scheduler"),
        }
    }
}

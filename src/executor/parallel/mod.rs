use hecs::World;
use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::SystemClosure;
use crate::{ArchetypeSet, BorrowSet, ExecutorBuilder, ResourceTuple, SystemId};

mod dispatching;
mod scheduling;

use dispatching::Dispatcher;
use scheduling::{DependantsLength, Scheduler};

static DISCONNECTED: &str = "channel should not be disconnected at this point";
static INVALID_ID: &str = "system IDs should always be valid";

/// System closure and scheduling metadata container.
pub struct System<'closure, Resources>
where
    Resources: ResourceTuple,
{
    pub closure: Arc<Mutex<SystemClosure<'closure, Resources::Wrapped>>>,
    pub resource_set: BorrowSet,
    pub component_set: BorrowSet,
    pub archetype_set: ArchetypeSet,
    pub archetype_writer: Box<dyn Fn(&World, &mut ArchetypeSet) + Send>,
    pub dependants: Vec<SystemId>,
    pub dependencies: usize,
    pub unsatisfied_dependencies: usize,
}

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
                        archetype_set: ArchetypeSet::default(),
                        archetype_writer: system.archetype_writer,
                        dependants: vec![],
                        dependencies,
                        unsatisfied_dependencies: 0,
                    },
                )
            })
            .collect();
        // If all systems are independent, it might be possible to use dispatching heuristic.
        if systems.len() == systems_without_dependencies.len() {
            let mut tested_ids = Vec::new();
            let mut all_disjoint = true;
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
            if all_disjoint {
                return ExecutorParallel::Dispatching(Dispatcher {
                    systems: systems
                        .drain()
                        .map(|(id, system)| (id, system.closure))
                        .collect(),
                });
            }
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
        ExecutorParallel::Scheduling(Scheduler {
            systems,
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

    pub fn force_archetype_recalculation(&mut self) {
        match self {
            ExecutorParallel::Dispatching(_) => (),
            ExecutorParallel::Scheduling(scheduler) => scheduler.archetypes_generation = None,
        }
    }

    pub fn run(&mut self, world: &World, wrapped: Resources::Wrapped) {
        match self {
            ExecutorParallel::Dispatching(dispatcher) => dispatcher.run(world, wrapped),
            ExecutorParallel::Scheduling(scheduler) => scheduler.run(world, wrapped),
        }
    }

    #[cfg(test)]
    fn unwrap_to_dispatcher(self) -> Dispatcher<'closures, Resources> {
        use ExecutorParallel::*;
        match self {
            Dispatching(dispatcher) => dispatcher,
            Scheduling(_) => panic!("produced executor is a scheduler"),
        }
    }

    #[cfg(test)]
    fn unwrap_to_scheduler(self) -> Scheduler<'closures, Resources> {
        use ExecutorParallel::*;
        match self {
            Dispatching(_) => panic!("produced executor is a dispatcher"),
            Scheduling(scheduler) => scheduler,
        }
    }
}

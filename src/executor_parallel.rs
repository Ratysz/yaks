use crossbeam_channel::{Receiver, Sender};
use hecs::ArchetypesGeneration;
use hecs::World;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    ArchetypeSet, ComponentSet, ExecutorBuilder, ResourceSet, ResourceTuple, ResourceWrap,
    SystemClosure, SystemContext, SystemId,
};

static DISCONNECTED: &str = "channel should not be disconnected at this point";
static INVALID_ID: &str = "system IDs should always be valid";

pub enum ExecutorParallel<'closures, Resources>
where
    Resources: ResourceTuple,
{
    Dispatching(Dispatcher<'closures, Resources>),
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
                    borrows: Resources::instantiate_borrows(),
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
            borrows: Resources::instantiate_borrows(),
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

    pub fn run<ResourceTuple>(&mut self, world: &World, resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        match self {
            ExecutorParallel::Dispatching(dispatcher) => dispatcher.run(world, resources),
            ExecutorParallel::Scheduling(scheduler) => scheduler.run(world, resources),
        }
    }
}

pub struct Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    borrows: Resources::Borrows,
    systems: HashMap<SystemId, Arc<Mutex<SystemClosure<'closures, Resources::Cells>>>>,
}

impl<'closures, Resources> Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        let wrapped = resources.wrap(&mut self.borrows);
        self.systems.par_iter().for_each(|(id, system)| {
            let system = &mut *system
                .try_lock() // TODO should this be .lock() instead?
                .expect("systems should only be ran once per execution");
            system(
                SystemContext {
                    system_id: Some(*id),
                    world,
                },
                &wrapped,
            );
        });
    }
}

struct System<'closure, Resources>
where
    Resources: ResourceTuple,
{
    closure: Arc<Mutex<SystemClosure<'closure, Resources::Cells>>>,
    resource_set: ResourceSet,
    component_set: ComponentSet,
    archetype_set: ArchetypeSet,
    archetype_writer: Box<dyn Fn(&World, &mut ArchetypeSet) + Send>,
    dependants: Vec<SystemId>,
    dependencies: usize,
    unsatisfied_dependencies: usize,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct DependantsLength(usize);

pub struct Scheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    borrows: Resources::Borrows,
    systems: HashMap<SystemId, System<'closures, Resources>>,
    archetypes_generation: Option<ArchetypesGeneration>,
    systems_without_dependencies: Vec<(SystemId, DependantsLength)>,
    systems_to_run_now: Vec<(SystemId, DependantsLength)>,
    systems_running: HashSet<SystemId>,
    systems_just_finished: Vec<SystemId>,
    systems_to_decrement_dependencies: Vec<SystemId>,
    sender: Sender<SystemId>,
    receiver: Receiver<SystemId>,
}

impl<'closures, Resources> Scheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        if Some(world.archetypes_generation()) == self.archetypes_generation {
            // If archetypes haven't changed since last run, reset dependency counters.
            for system in self.systems.values_mut() {
                debug_assert!(system.unsatisfied_dependencies == 0);
                system.unsatisfied_dependencies = system.dependencies;
            }
        } else {
            // If archetypes have changed, recalculate archetype sets for all systems,
            // and reset dependency counters.
            self.archetypes_generation = Some(world.archetypes_generation());
            for system in self.systems.values_mut() {
                (system.archetype_writer)(world, &mut system.archetype_set);
                debug_assert!(system.unsatisfied_dependencies == 0);
                system.unsatisfied_dependencies = system.dependencies;
            }
        }
        // Queue systems that don't have any dependencies to run first.
        self.systems_to_run_now
            .extend(&self.systems_without_dependencies);
        // Wrap resources for disjoint fetching.
        let wrapped = resources.wrap(&mut self.borrows);
        let wrapped = &wrapped;
        rayon::scope(|scope| {
            // All systems have been ran if there are no queued or currently running systems.
            while !(self.systems_to_run_now.is_empty() && self.systems_running.is_empty()) {
                for (id, _) in &self.systems_to_run_now {
                    // Check if a queued system can run concurrently with
                    // other systems already running.
                    if self.can_run_now(*id) {
                        // Add it to the currently running systems set.
                        self.systems_running.insert(*id);
                        // Pointers and data sent over to a worker thread.
                        let system = self.systems.get_mut(id).expect(INVALID_ID).closure.clone();
                        let sender = self.sender.clone();
                        let id = *id;
                        scope.spawn(move |_| {
                            let system = &mut *system
                                .try_lock() // TODO should this be .lock() instead?
                                .expect("systems should only be ran once per execution");
                            system(
                                SystemContext {
                                    system_id: Some(id),
                                    world,
                                },
                                wrapped,
                            );
                            // Notify dispatching thread than this system has finished running.
                            sender.send(id).expect(DISCONNECTED);
                        });
                    }
                }
                {
                    // Remove newly running systems from systems-to-run-now set.
                    // TODO replace with `.drain_filter()` once stable
                    //  https://github.com/rust-lang/rust/issues/43244
                    let mut i = 0;
                    while i != self.systems_to_run_now.len() {
                        if self.systems_running.contains(&self.systems_to_run_now[i].0) {
                            self.systems_to_run_now.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
                // Wait until at least one system is finished.
                let id = self.receiver.recv().expect(DISCONNECTED);
                self.systems_just_finished.push(id);
                // Handle any other systems that may have finished.
                self.systems_just_finished.extend(self.receiver.try_iter());
                // Remove finished systems from set of running systems.
                for id in &self.systems_just_finished {
                    self.systems_running.remove(id);
                }
                // Gather dependants of finished systems.
                for finished in &self.systems_just_finished {
                    for dependant in &self.systems.get(finished).expect(INVALID_ID).dependants {
                        self.systems_to_decrement_dependencies.push(*dependant);
                    }
                }
                self.systems_just_finished.clear();
                // Figure out which of the gathered dependants have had all their dependencies
                // satisfied and queue them to run.
                for id in &self.systems_to_decrement_dependencies {
                    let system = &mut self.systems.get_mut(id).expect(INVALID_ID);
                    let dependants = DependantsLength(system.dependants.len());
                    let unsatisfied_dependencies = &mut system.unsatisfied_dependencies;
                    *unsatisfied_dependencies -= 1;
                    if *unsatisfied_dependencies == 0 {
                        self.systems_to_run_now.push((*id, dependants));
                    }
                }
                self.systems_to_decrement_dependencies.clear();
                // Sort queued systems so that those with most dependants run first.
                self.systems_to_run_now.sort_by(|(_, a), (_, b)| b.cmp(a));
            }
        });
        debug_assert!(self.systems_to_run_now.is_empty());
        debug_assert!(self.systems_running.is_empty());
        debug_assert!(self.systems_just_finished.is_empty());
        debug_assert!(self.systems_to_decrement_dependencies.is_empty());
    }

    fn can_run_now(&self, id: SystemId) -> bool {
        let system = self.systems.get(&id).expect(INVALID_ID);
        for id in &self.systems_running {
            let running_system = self.systems.get(id).expect(INVALID_ID);
            // A system can't run if the resources it needs are already borrowed incompatibly.
            if !system
                .resource_set
                .is_compatible(&running_system.resource_set)
            {
                return false;
            }
            // A system can't run if it could borrow incompatibly any components.
            // This can only happen if the system could incompatibly access the same components
            // from the same archetype that another system may be using.
            if !system
                .component_set
                .is_compatible(&running_system.component_set)
                && !system
                    .archetype_set
                    .is_compatible(&running_system.archetype_set)
            {
                return false;
            }
        }
        true
    }
}

#[test]
fn dispatch_heuristic_trivial() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<()>::build(
        ExecutorBuilder::new()
            .system(|_, _: (), _: ()| {})
            .system(|_, _: (), _: ()| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_trivial_with_resources() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<(usize, f32)>::build(
        ExecutorBuilder::new()
            .system(|_, _: (), _: ()| {})
            .system(|_, _: (), _: ()| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_resources_incompatible() {
    if let ExecutorParallel::Dispatching(_) = ExecutorParallel::<(usize, f32)>::build(
        ExecutorBuilder::new()
            .system(|_, _: &f32, _: ()| {})
            .system(|_, _: &mut f32, _: ()| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_resources_disjoint() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<(usize, f32)>::build(
        ExecutorBuilder::new()
            .system(|_, _: &mut usize, _: ()| {})
            .system(|_, _: &mut f32, _: ()| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_resources_immutable() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<(usize, f32)>::build(
        ExecutorBuilder::new()
            .system(|_, _: &f32, _: ()| {})
            .system(|_, _: &f32, _: ()| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_queries_incompatible() {
    if let ExecutorParallel::Dispatching(_) = ExecutorParallel::<()>::build(
        ExecutorBuilder::new()
            .system(|_, _: (), _: crate::QueryMarker<&mut f32>| {})
            .system(|_, _: (), _: crate::QueryMarker<&f32>| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_queries_disjoint() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<()>::build(
        ExecutorBuilder::new()
            .system(|_, _: (), _: crate::QueryMarker<&mut usize>| {})
            .system(|_, _: (), _: crate::QueryMarker<&mut f32>| {}),
    ) {
        panic!();
    }
}

#[test]
fn dispatch_heuristic_queries_immutable() {
    if let ExecutorParallel::Scheduling(_) = ExecutorParallel::<()>::build(
        ExecutorBuilder::new()
            .system(|_, _: (), _: crate::QueryMarker<&f32>| {})
            .system(|_, _: (), _: crate::QueryMarker<&f32>| {}),
    ) {
        panic!();
    }
}

use crossbeam_channel::{Receiver, Sender};
use hecs::{ArchetypesGeneration, World};
use std::collections::{HashMap, HashSet};

use super::{System, DISCONNECTED, INVALID_ID};
use crate::{ResourceTuple, ResourceWrap, SystemContext, SystemId};

/// Typed `usize` used to cache the amount of dependants the system associated
/// with a `SystemId` has; avoids hashmap lookups while sorting.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct DependantsLength(pub usize);

/// Parallel executor variant, used when systems cannot be proven to be statically disjoint,
/// or have dependencies.
pub struct Scheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub borrows: Resources::Borrows,
    pub systems: HashMap<SystemId, System<'closures, Resources>>,
    pub archetypes_generation: Option<ArchetypesGeneration>,
    pub systems_without_dependencies: Vec<(SystemId, DependantsLength)>,
    pub systems_to_run_now: Vec<(SystemId, DependantsLength)>,
    pub systems_running: HashSet<SystemId>,
    pub systems_just_finished: Vec<SystemId>,
    pub systems_to_decrement_dependencies: Vec<SystemId>,
    pub sender: Sender<SystemId>,
    pub receiver: Receiver<SystemId>,
}

impl<'closures, Resources> Scheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
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

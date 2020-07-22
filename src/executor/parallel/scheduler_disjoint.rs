use crossbeam_channel::{Receiver, Sender};
use hecs::{ArchetypesGeneration, World};
use parking_lot::Mutex;
use rayon::ScopeFifo;
use std::{collections::HashMap, sync::Arc};

use super::{DependantsLength, SystemClosure, DISCONNECTED, INVALID_ID};
use crate::{ResourceTuple, SystemContext, SystemId};

pub struct System<'closure, Resources>
where
    Resources: ResourceTuple,
{
    pub closure: Arc<Mutex<SystemClosure<'closure, Resources::Wrapped>>>,
    pub dependants: Vec<SystemId>,
    pub dependencies: usize,
    pub unsatisfied_dependencies: usize,
}

pub struct DisjointScheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub systems: HashMap<SystemId, System<'closures, Resources>>,
    pub archetypes_generation: Option<ArchetypesGeneration>,
    pub systems_without_dependencies: Vec<(SystemId, DependantsLength)>,
    pub systems_to_run_now: Vec<(SystemId, DependantsLength)>,
    pub systems_running: usize,
    pub systems_just_finished: Vec<SystemId>,
    pub systems_to_decrement_dependencies: Vec<SystemId>,
    pub sender: Sender<SystemId>,
    pub receiver: Receiver<SystemId>,
}

impl<'closures, Resources> DisjointScheduler<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn run(&mut self, world: &World, wrapped: Resources::Wrapped) {
        debug_assert!(self.systems_to_run_now.is_empty());
        debug_assert!(self.systems_running == 0);
        debug_assert!(self.systems_just_finished.is_empty());
        debug_assert!(self.systems_to_decrement_dependencies.is_empty());
        rayon::scope_fifo(|scope| {
            self.prepare();
            // All systems have been ran if there are no queued or currently running systems.
            while !(self.systems_to_run_now.is_empty() && self.systems_running == 0) {
                self.start_all_currently_runnable(scope, world, &wrapped);
                self.wait_for_and_process_finished();
            }
        });
        debug_assert!(self.systems_to_run_now.is_empty());
        debug_assert!(self.systems_running == 0);
        debug_assert!(self.systems_just_finished.is_empty());
        debug_assert!(self.systems_to_decrement_dependencies.is_empty());
    }

    fn prepare(&mut self) {
        // Queue systems that don't have any dependencies to run first.
        self.systems_to_run_now
            .extend(&self.systems_without_dependencies);
        // Reset dependency counters.
        for system in self.systems.values_mut() {
            debug_assert!(system.unsatisfied_dependencies == 0);
            system.unsatisfied_dependencies = system.dependencies;
        }
    }

    fn start_all_currently_runnable<'run>(
        &mut self,
        scope: &ScopeFifo<'run>,
        world: &'run World,
        wrapped: &'run Resources::Wrapped,
    ) where
        'closures: 'run,
        Resources::BorrowTuple: Send,
        Resources::Wrapped: Send + Sync,
    {
        for (id, _) in self.systems_to_run_now.drain(..) {
            self.systems_running += 1;
            // Pointers and data to send over to a worker thread.
            let system = self.systems.get_mut(&id).expect(INVALID_ID).closure.clone();
            let sender = self.sender.clone();
            scope.spawn_fifo(move |_| {
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
                // Notify dispatching thread that this system has finished running.
                sender.send(id).expect(DISCONNECTED);
            });
        }
    }

    fn wait_for_and_process_finished(&mut self) {
        // Wait until at least one system is finished.
        self.systems_just_finished
            .push(self.receiver.recv().expect(DISCONNECTED));
        // Handle any other systems that may have finished.
        self.systems_just_finished.extend(self.receiver.try_iter());
        for finished in self.systems_just_finished.drain(..) {
            self.systems_running -= 1;
            // Gather dependants of the finished system.
            for dependant in &self.systems.get(&finished).expect(INVALID_ID).dependants {
                self.systems_to_decrement_dependencies.push(*dependant);
            }
        }
        // Figure out which of the gathered dependants have had all their dependencies
        // satisfied and queue them to run.
        for id in self.systems_to_decrement_dependencies.drain(..) {
            let system = &mut self.systems.get_mut(&id).expect(INVALID_ID);
            let unsatisfied_dependencies = &mut system.unsatisfied_dependencies;
            *unsatisfied_dependencies -= 1;
            if *unsatisfied_dependencies == 0 {
                self.systems_to_run_now
                    .push((id, DependantsLength(system.dependants.len())));
            }
        }
        // Sort queued systems so that those with most dependants run first.
        self.systems_to_run_now.sort_by(|(_, a), (_, b)| b.cmp(a));
    }

    /*#[cfg(test)]
    fn wait_for_one_finished(&mut self) {
        self.systems_just_finished
            .push(self.receiver.recv().expect(DISCONNECTED));
    }*/
}

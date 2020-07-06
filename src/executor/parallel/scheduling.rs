use crossbeam_channel::{Receiver, Sender};
use hecs::{ArchetypesGeneration, World};
use rayon::ScopeFifo;
use std::collections::{HashMap, HashSet};

use super::{System, DISCONNECTED, INVALID_ID};
use crate::{ResourceTuple, SystemContext, SystemId};

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
    pub fn run(&mut self, world: &World, wrapped: Resources::Wrapped) {
        rayon::scope_fifo(|scope| {
            self.prepare(world);
            // All systems have been ran if there are no queued or currently running systems.
            while !(self.systems_to_run_now.is_empty() && self.systems_running.is_empty()) {
                self.start_all_currently_runnable(scope, world, &wrapped);
                self.wait_for_and_process_finished();
            }
        });
        debug_assert!(self.systems_to_run_now.is_empty());
        debug_assert!(self.systems_running.is_empty());
        debug_assert!(self.systems_just_finished.is_empty());
        debug_assert!(self.systems_to_decrement_dependencies.is_empty());
    }

    fn prepare(&mut self, world: &World) {
        debug_assert!(self.systems_to_run_now.is_empty());
        debug_assert!(self.systems_running.is_empty());
        debug_assert!(self.systems_just_finished.is_empty());
        debug_assert!(self.systems_to_decrement_dependencies.is_empty());
        // Queue systems that don't have any dependencies to run first.
        self.systems_to_run_now
            .extend(&self.systems_without_dependencies);
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
        for (id, _) in &self.systems_to_run_now {
            // Check if a queued system can run concurrently with
            // other systems already running.
            if self.can_start_now(*id) {
                // Add it to the currently running systems set.
                self.systems_running.insert(*id);
                // Pointers and data to send over to a worker thread.
                let system = self.systems.get_mut(id).expect(INVALID_ID).closure.clone();
                let sender = self.sender.clone();
                let id = *id;
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
    }

    fn can_start_now(&self, id: SystemId) -> bool {
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

    fn wait_for_and_process_finished(&mut self) {
        // Wait until at least one system is finished.
        self.systems_just_finished
            .push(self.receiver.recv().expect(DISCONNECTED));
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
        for id in self.systems_to_decrement_dependencies.drain(..) {
            let system = &mut self.systems.get_mut(&id).expect(INVALID_ID);
            let dependants = DependantsLength(system.dependants.len());
            let unsatisfied_dependencies = &mut system.unsatisfied_dependencies;
            *unsatisfied_dependencies -= 1;
            if *unsatisfied_dependencies == 0 {
                self.systems_to_run_now.push((id, dependants));
            }
        }
        // Sort queued systems so that those with most dependants run first.
        self.systems_to_run_now.sort_by(|(_, a), (_, b)| b.cmp(a));
    }

    #[cfg(test)]
    fn wait_for_one_finished(&mut self) {
        self.systems_just_finished
            .push(self.receiver.recv().expect(DISCONNECTED));
    }
}

#[cfg(test)]
mod tests {
    use super::super::ExecutorParallel;
    use crate::{
        resource::{AtomicBorrow, ResourceWrap},
        Executor, QueryMarker, SystemContext,
    };
    use hecs::World;
    use rayon::{ScopeFifo, ThreadPoolBuilder};

    struct A(usize);
    struct B(usize);
    struct C(usize);

    fn dummy_system(_: SystemContext, _: (), _: ()) {}

    fn local_pool_scope_fifo<'scope, F>(closure: F)
    where
        F: for<'s> FnOnce(&'s ScopeFifo<'scope>) + 'scope + Send,
    {
        ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap()
            .scope_fifo(closure)
    }

    #[test]
    fn dependencies_single() {
        let world = World::new();
        let mut executor = ExecutorParallel::<()>::build(
            Executor::builder()
                .system_with_handle(dummy_system, 0)
                .system_with_handle_and_deps(dummy_system, 1, vec![0]),
        )
        .unwrap_to_scheduler();
        let wrapped = ();
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
    }

    #[test]
    fn dependencies_several() {
        let world = World::new();
        let mut executor = ExecutorParallel::<()>::build(
            Executor::<()>::builder()
                .system_with_handle(dummy_system, 0)
                .system_with_handle(dummy_system, 1)
                .system_with_handle(dummy_system, 2)
                .system_with_deps(dummy_system, vec![0, 1, 2]),
        )
        .unwrap_to_scheduler();
        let wrapped = ();
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 3);
            executor.wait_for_one_finished();
            executor.wait_for_one_finished();
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
    }

    #[test]
    fn dependencies_chain() {
        let world = World::new();
        let mut executor = ExecutorParallel::<()>::build(
            Executor::<()>::builder()
                .system_with_handle(dummy_system, 0)
                .system_with_handle_and_deps(dummy_system, 1, vec![0])
                .system_with_handle_and_deps(dummy_system, 2, vec![1])
                .system_with_deps(dummy_system, vec![2]),
        )
        .unwrap_to_scheduler();
        let wrapped = ();
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
    }

    #[test]
    fn dependencies_fully_constrained() {
        let world = World::new();
        let mut executor = ExecutorParallel::<()>::build(
            Executor::<()>::builder()
                .system_with_handle(dummy_system, 0)
                .system_with_handle_and_deps(dummy_system, 1, vec![0])
                .system_with_handle_and_deps(dummy_system, 2, vec![0, 1])
                .system_with_deps(dummy_system, vec![0, 1, 2]),
        )
        .unwrap_to_scheduler();
        let wrapped = ();
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
    }

    #[test]
    fn resources_incompatible_mutable_immutable() {
        let world = World::new();
        let mut executor = ExecutorParallel::<(A,)>::build(
            Executor::builder()
                .system(|_, _: &A, _: ()| {})
                .system(|_, a: &mut A, _: ()| a.0 += 1),
        )
        .unwrap_to_scheduler();
        let mut a = A(0);
        let mut a = &mut a;
        let mut borrows = (AtomicBorrow::new(),);
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        assert_eq!(a.0, 1);
    }

    #[test]
    fn resources_incompatible_mutable_mutable() {
        let world = World::new();
        let mut executor = ExecutorParallel::<(A,)>::build(
            Executor::builder()
                .system(|_, a: &mut A, _: ()| a.0 += 1)
                .system(|_, a: &mut A, _: ()| a.0 += 1),
        )
        .unwrap_to_scheduler();
        let mut a = A(0);
        let mut a = &mut a;
        let mut borrows = (AtomicBorrow::new(),);
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        assert_eq!(a.0, 2);
    }

    #[test]
    fn queries_incompatible_mutable_immutable() {
        let mut world = World::new();
        world.spawn_batch((0..10).map(|_| (B(0),)));
        let mut executor = ExecutorParallel::<(A,)>::build(
            Executor::builder()
                .system(|ctx, _: (), q: QueryMarker<&B>| for (_, _) in ctx.query(q).iter() {})
                .system(|ctx, a: &A, q: QueryMarker<&mut B>| {
                    for (_, b) in ctx.query(q).iter() {
                        b.0 += a.0;
                    }
                }),
        )
        .unwrap_to_scheduler();
        let mut a = A(1);
        let mut a = &mut a;
        let mut borrows = (AtomicBorrow::new(),);
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        for (_, b) in world.query::<&B>().iter() {
            assert_eq!(b.0, 1);
        }
    }

    #[test]
    fn queries_incompatible_mutable_mutable() {
        let mut world = World::new();
        world.spawn_batch((0..10).map(|_| (B(0),)));
        let mut executor = ExecutorParallel::<(A,)>::build(
            Executor::builder()
                .system(|ctx, a: &A, q: QueryMarker<&mut B>| {
                    for (_, b) in ctx.query(q).iter() {
                        b.0 += a.0;
                    }
                })
                .system(|ctx, a: &A, q: QueryMarker<&mut B>| {
                    for (_, b) in ctx.query(q).iter() {
                        b.0 += a.0;
                    }
                }),
        )
        .unwrap_to_scheduler();
        let mut a = A(1);
        let mut a = &mut a;
        let mut borrows = (AtomicBorrow::new(),);
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        for (_, b) in world.query::<&B>().iter() {
            assert_eq!(b.0, 2);
        }
    }

    #[test]
    fn queries_disjoint_by_archetypes() {
        let mut world = World::new();
        world.spawn_batch((0..10).map(|_| (A(0), B(0))));
        world.spawn_batch((0..10).map(|_| (B(0), C(0))));
        let mut executor = ExecutorParallel::<(A,)>::build(
            Executor::builder()
                .system(|ctx, a: &A, q: QueryMarker<(&A, &mut B)>| {
                    for (_, (_, b)) in ctx.query(q).iter() {
                        b.0 += a.0;
                    }
                })
                .system(|ctx, a: &A, q: QueryMarker<(&mut B, &C)>| {
                    for (_, (b, _)) in ctx.query(q).iter() {
                        b.0 += a.0;
                    }
                }),
        )
        .unwrap_to_scheduler();
        let mut a = A(2);
        let mut a = &mut a;
        let mut borrows = (AtomicBorrow::new(),);
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 2);
            executor.wait_for_one_finished();
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        for (_, b) in world.query::<&B>().iter() {
            assert_eq!(b.0, 2);
        }

        /*let mut entities: Vec<_> = world
        .spawn_batch((0..10).map(|_| (A(0), B(1), C(0))))
        .collect();*/
        world.spawn_batch((0..10).map(|_| (A(0), B(1), C(0))));
        let mut a = A(1);
        let mut a = &mut a;
        let wrapped = a.wrap(&mut borrows);
        local_pool_scope_fifo(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());

            executor.start_all_currently_runnable(scope, &world, &wrapped);
            assert_eq!(executor.systems_running.len(), 1);
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        for (_, b) in world.query::<&B>().iter() {
            assert_eq!(b.0, 3);
        }

        /*entities
            .drain(..)
            .for_each(|entity| world.despawn(entity).unwrap());
        rayon::scope(|scope| {
            executor.prepare(&world);
            executor.start_all_currently_runnable(scope, &world, &wrapped);
            // TODO this fails. Suggest upstream changes?
            assert_eq!(executor.systems_running.len(), 2);
            executor.wait_for_one_finished();
            executor.wait_for_and_process_finished();
            assert!(executor.systems_running.is_empty());
            assert!(executor.systems_to_run_now.is_empty());
        });
        for (_, b) in world.query::<&B>().iter() {
            assert_eq!(b.0, 4);
        }*/
    }
}

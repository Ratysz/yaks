use hecs::World;
use resources::Resources;
use std::hash::Hash;

use crate::{Executor, ModQueuePool};

impl<H> Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn maintain_parallelization_data(&mut self) {
        self.all_resources.clear();
        self.all_components.clear();
        for borrows_container in self.borrows.values() {
            self.all_resources
                .extend(&borrows_container.borrows.resources_mutable);
            self.all_resources
                .extend(&borrows_container.borrows.resources_immutable);
            self.all_components
                .extend(&borrows_container.borrows.components_mutable);
            self.all_components
                .extend(&borrows_container.borrows.components_immutable);
        }
        for borrows_container in self.borrows.values_mut() {
            borrows_container.condensed = borrows_container
                .borrows
                .condense(&self.all_resources, &self.all_components);
        }
    }

    fn can_run_now(&self, system_to_run_index: usize) -> bool {
        let index = self.systems_to_run[system_to_run_index];
        for dependency in &self.system_container(index).dependencies {
            if !self.finished_systems.contains(
                &self
                    .index(dependency)
                    .unwrap_or_else(|error| panic!("{}", error)),
            ) {
                return false;
            }
        }
        let borrows_container = self.borrows_container(index);
        for current_borrows in self
            .current_systems
            .iter()
            .map(|index| self.borrows_container(*index))
        {
            if !borrows_container
                .condensed
                .are_resources_compatible(&current_borrows.condensed)
            {
                return false;
            }
            if !borrows_container
                .condensed
                .are_components_compatible(&current_borrows.condensed)
            {
                // TODO archetypes
                return false;
            }
        }
        true
    }

    pub fn run_parallel<'scope, S>(
        &'scope mut self,
        world: &'scope World,
        resources: &'scope Resources,
        mod_queues: &'scope ModQueuePool,
        scope: &S,
    ) where
        S: ThreadpoolScope<'scope>,
    {
        if self.dirty {
            self.maintain();
            self.maintain_parallelization_data();
        }
        self.systems_to_run.clear();
        self.current_systems.clear();
        self.finished_systems.clear();
        for index in &self.systems_sorted {
            if self.system_container(*index).active {
                self.systems_to_run.push(*index);
            }
        }
        while !self.systems_to_run.is_empty() {
            for i in 0..self.systems_to_run.len() {
                if self.can_run_now(i) {
                    let index = self.systems_to_run[i];
                    let system = self.system_container_mut(index).system.clone();
                    let sender = self.sender.clone();
                    scope.execute(move || {
                        system
                            .lock()
                            .expect("mutexes should never be poisoned")
                            .run(world, resources, mod_queues);
                        sender
                            .send(index)
                            .expect("channel should not be disconnected at this point");
                    });
                    self.current_systems.insert(index);
                }
            }
            {
                // Remove newly running systems from systems-to-run.
                // TODO replace with `.drain_filter()` once stable
                //  https://github.com/rust-lang/rust/issues/43244
                let mut i = 0;
                while i != self.systems_to_run.len() {
                    if self.current_systems.contains(&self.systems_to_run[i]) {
                        self.systems_to_run.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            // Wait until at least one system is finished.
            let index = self
                .receiver
                .recv()
                .expect("channel should not be disconnected at this point");
            self.finished_systems.insert(index);
            self.current_systems.remove(&index);
            // Process any other systems that have finished.
            while !self.receiver.is_empty() {
                let index = self
                    .receiver
                    .recv()
                    .expect("channel should not be disconnected at this point");
                self.finished_systems.insert(index);
                self.current_systems.remove(&index);
            }
        }
    }
}

pub trait ThreadpoolScope<'scope> {
    fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'scope;
}

#[cfg(feature = "impl_scoped_threadpool")]
impl<'pool, 'scope> ThreadpoolScope<'scope> for scoped_threadpool::Scope<'pool, 'scope> {
    fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        self.execute(closure);
    }
}

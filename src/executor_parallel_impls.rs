use hecs::World;
use resources::Resources;
use std::{fmt::Debug, hash::Hash};

use crate::{
    executor::{SystemIndex, INVALID_INDEX},
    threadpool::DISCONNECTED,
    Executor, ModQueuePool, Scope,
};

impl<H> Executor<H>
where
    H: Hash + Eq + PartialEq + Debug,
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

    fn can_run_now(&self, index: SystemIndex) -> bool {
        for dependency in &self.systems.get(&index).expect(INVALID_INDEX).dependencies {
            if !self.finished_systems.contains(
                &self
                    .resolve_handle(dependency)
                    .expect("all dependencies should have been validated by this point"),
            ) {
                return false;
            }
        }
        let borrows_container = self.borrows.get(&index).expect(INVALID_INDEX);
        for current_borrows in self
            .current_systems
            .iter()
            .map(|index| self.borrows.get(index).expect(INVALID_INDEX))
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
                && !borrows_container
                    .archetypes
                    .as_bitset()
                    .is_disjoint(&current_borrows.archetypes.as_bitset())
            {
                return false;
            }
        }
        true
    }

    pub fn run_parallel<'scope>(
        &mut self,
        world: &'scope World,
        resources: &'scope Resources,
        mod_queues: &'scope ModQueuePool,
        scope: &Scope<'scope>,
    ) {
        if self.dirty {
            self.maintain_parallelization_data();
            self.dirty = false;
        }
        self.systems_to_run.clear();
        self.current_systems.clear();
        self.finished_systems.clear();
        for i in 0..self.systems_sorted.len() {
            let index = self.systems_sorted[i];
            let system_container = self.systems.get_mut(&index).expect(INVALID_INDEX);
            if system_container.active {
                let borrows_container = self.borrows.get_mut(&index).expect(INVALID_INDEX);
                borrows_container.archetypes.clear();
                system_container
                    .system_mut()
                    .inner()
                    .write_archetypes(world, &mut borrows_container.archetypes);
                self.systems_to_run.push(index);
            }
        }
        while !self.systems_to_run.is_empty() {
            for i in 0..self.systems_to_run.len() {
                let index = self.systems_to_run[i];
                if self.can_run_now(index) {
                    let system = self
                        .systems
                        .get_mut(&index)
                        .expect(INVALID_INDEX)
                        .clone_arc();
                    let sender = self.sender.clone();
                    scope.execute(move || {
                        system
                            .lock()
                            .expect("mutexes should never be poisoned")
                            .run_with_scope(world, resources, mod_queues, scope);
                        sender.send(index).expect(DISCONNECTED);
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
            let index = self.receiver.recv().expect(DISCONNECTED);
            self.finished_systems.insert(index);
            self.current_systems.remove(&index);
            // Process any other systems that have finished.
            while !self.receiver.is_empty() {
                let index = self.receiver.recv().expect(DISCONNECTED);
                self.finished_systems.insert(index);
                self.current_systems.remove(&index);
            }
        }
    }
}

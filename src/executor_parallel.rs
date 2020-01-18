use crossbeam::channel::{self, Receiver, Sender};
use fxhash::FxHasher64;
use hecs::World;
use resources::Resources;
use std::{
    collections::HashSet,
    hash::{BuildHasherDefault, Hash},
    ops::{Deref, DerefMut},
};

use crate::{borrows::TypeSet, executor::SystemIndex, Executor, ModQueuePool, Scope, Threadpool};

pub struct ParallelExecutor<H>
where
    H: Hash + Eq + PartialEq,
{
    executor: Executor<H>,
    all_resources: TypeSet,
    all_components: TypeSet,
    systems_to_run: Vec<SystemIndex>,
    current_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
    finished_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
    sender: Sender<SystemIndex>,
    receiver: Receiver<SystemIndex>,
}

impl<H> Default for ParallelExecutor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        let (sender, receiver) = channel::bounded(1);
        Self {
            executor: Default::default(),
            all_resources: Default::default(),
            all_components: Default::default(),
            systems_to_run: Default::default(),
            current_systems: Default::default(),
            finished_systems: Default::default(),
            sender,
            receiver,
        }
    }
}

impl<H> ParallelExecutor<H>
where
    H: Hash + Eq + PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_executor(executor: Executor<H>) -> Self {
        Self {
            executor,
            ..Default::default()
        }
    }

    pub(crate) fn maintain(&mut self) {
        self.executor.maintain();
        self.all_resources.clear();
        self.all_components.clear();
        for container in self.executor.systems.values() {
            self.all_resources
                .extend(&container.borrows.resources_mutable);
            self.all_resources
                .extend(&container.borrows.resources_immutable);
            self.all_components
                .extend(&container.borrows.components_mutable);
            self.all_components
                .extend(&container.borrows.components_immutable);
        }
        for container in self.executor.systems.values_mut() {
            container.condensed = container
                .borrows
                .condense(&self.all_resources, &self.all_components);
        }
    }

    fn can_run_now(&self, system_to_run_index: usize) -> bool {
        let container = self
            .executor
            .systems
            .get(&self.systems_to_run[system_to_run_index])
            .expect("this key should be present at this point");
        for dependency in &container.dependencies {
            if !self.finished_systems.contains(
                self.executor
                    .system_handles
                    .get(dependency)
                    .expect("system handles should always map to valid system indices"),
            ) {
                return false;
            }
        }
        for current in self.current_systems.iter().map(|index| {
            self.executor
                .systems
                .get(index)
                .expect("this key should be present at this point")
        }) {
            if !container
                .condensed
                .are_resources_compatible(&current.condensed)
            {
                return false;
            }
            if !container
                .condensed
                .are_components_compatible(&current.condensed)
            {
                // TODO archetypes
                return false;
            }
        }
        true
    }

    pub fn run_parallel<'pool, 'scope, P, S>(
        &'pool mut self,
        world: &'scope World,
        resources: &'scope Resources,
        mod_queues: &'scope ModQueuePool,
        threadpool: &'pool mut P,
    ) where
        'pool: 'scope,
        P: Threadpool<'pool, 'scope, S>,
        S: Scope<'pool, 'scope>,
    {
        if self.executor.dirty {
            self.maintain();
        }
        self.systems_to_run.clear();
        self.current_systems.clear();
        self.finished_systems.clear();
        for index in &self.executor.systems_sorted {
            if self
                .executor
                .systems
                .get(index)
                .expect("this key should be present at this point")
                .active
            {
                self.systems_to_run.push(*index);
            }
        }
        threadpool.scope(|threadpool| {
            while !self.systems_to_run.is_empty() {
                for i in 0..self.systems_to_run.len() {
                    if self.can_run_now(i) {
                        let index = self.systems_to_run[i];
                        let system = self
                            .executor
                            .systems
                            .get_mut(&index)
                            .expect("this key should be present at this point")
                            .system
                            .clone();
                        let sender = self.sender.clone();
                        threadpool.execute(move || {
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
                // Process however more systems that have finished.
                while !self.receiver.is_empty() {
                    let index = self
                        .receiver
                        .recv()
                        .expect("channel should not be disconnected at this point");
                    self.finished_systems.insert(index);
                    self.current_systems.remove(&index);
                }
            }
        });
    }
}

impl<H> Deref for ParallelExecutor<H>
where
    H: Hash + Eq + PartialEq,
{
    type Target = Executor<H>;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}

impl<H> DerefMut for ParallelExecutor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.executor
    }
}

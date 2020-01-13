use fxhash::FxHasher64;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

use crate::{
    error::NoSuchSystem,
    //system::{ArchetypeSet, SystemBorrows},
    System,
    World,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex(usize);

struct SystemContainer {
    system: System,
    active: bool,
    //borrows: SystemBorrows,
    //archetypes: ArchetypeSet,
}

impl SystemContainer {
    fn new(system: System) -> Self {
        /*let mut borrows = SystemBorrows::default();
        system.write_borrows(&mut borrows);
        let archetypes = ArchetypeSet::default();*/
        Self {
            system,
            active: true,
            //borrows,
            //archetypes,
        }
    }
}

pub struct Executor<H = ()>
where
    H: Hash + Eq + PartialEq,
{
    systems: HashMap<SystemIndex, SystemContainer, BuildHasherDefault<FxHasher64>>,
    system_handles: HashMap<H, SystemIndex>,
    free_indices: Vec<SystemIndex>,
}

impl<H> Default for Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H> Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    pub fn new() -> Self {
        Self {
            systems: Default::default(),
            free_indices: Default::default(),
            system_handles: HashMap::default(),
        }
    }

    fn new_system_index(&mut self) -> SystemIndex {
        if let Some(index) = self.free_indices.pop() {
            index
        } else {
            SystemIndex(self.systems.len())
        }
    }

    pub fn run(&mut self, world: &mut World) {
        self.systems
            .values_mut()
            .filter(|container| container.active)
            .for_each(|container| container.system.run(world));
        world.flush_mod_queues();
    }

    //fn add_inner(&mut self, dependencies: &[H], system: System) -> SystemIndex {
    fn add_inner(&mut self, system: System) -> SystemIndex {
        let container = SystemContainer::new(system);
        let index = self.new_system_index();
        self.systems.insert(index, container);
        index
    }

    pub fn add(&mut self, system: System) {
        self.add_inner(system);
    }

    pub fn with(mut self, system: System) -> Self {
        self.add(system);
        self
    }

    fn add_with_handle_inner(&mut self, handle: H, system: System) -> Option<System> {
        let container = SystemContainer::new(system);
        let index = self
            .system_handles
            .get(&handle)
            .copied()
            .unwrap_or_else(|| {
                let index = self.new_system_index();
                self.system_handles.insert(handle, index);
                index
            });
        self.systems
            .insert(index, container)
            .map(|container| container.system)
    }

    pub fn add_with_handle(&mut self, handle: H, system: System) -> Option<System> {
        self.add_with_handle_inner(handle, system)
    }

    pub fn with_handle(mut self, handle: H, system: System) -> Self {
        self.add_with_handle(handle, system);
        self
    }

    pub fn is_active(&self, handle: &H) -> Result<bool, NoSuchSystem> {
        match self.system_handles.get(handle) {
            Some(index) => Ok(self
                .systems
                .get(index)
                .expect("system handles should always map to valid system indices")
                .active),
            None => Err(NoSuchSystem),
        }
    }

    pub fn set_active(&mut self, handle: &H, active: bool) -> Result<(), NoSuchSystem> {
        match self.system_handles.get_mut(handle) {
            Some(index) => {
                self.systems
                    .get_mut(index)
                    .expect("system handles should always map to valid system indices")
                    .active = active;
                Ok(())
            }
            None => Err(NoSuchSystem),
        }
    }

    /*pub fn add(&mut self, dependencies: &[H], system: System) {
        self.add_inner(dependencies, system);
    }

    pub fn with(mut self, dependencies: &[H], system: System) -> Self {
        self.add(dependencies, system);
        self
    }

    #[allow(clippy::map_entry)]
    fn add_with_handle_inner(
        &mut self,
        handle: H,
        dependencies: &[H],
        system: System,
    ) -> Result<(), SystemHandleIsNotUnique> {
        if self.system_handles.contains_key(&handle) {
            Err(SystemHandleIsNotUnique)
        } else {
            let index = self.add_inner(dependencies, system);
            self.system_handles.insert(handle, index);
            Ok(())
        }
    }

    pub fn add_with_handle(
        &mut self,
        handle: H,
        dependencies: &[H],
        system: System,
    ) -> Result<(), SystemHandleIsNotUnique> {
        self.add_with_handle_inner(handle, dependencies, system)
    }

    pub fn with_handle(mut self, handle: H, dependencies: &[H], system: System) -> Self {
        if let Err(error) = self.add_with_handle(handle, dependencies, system) {
            panic!("{}", error);
        }
        self
    }*/

    /*pub fn run_parallel(&mut self, world: &mut World, pool: &mut scoped_threadpool::Pool) {
        pool.scoped(|scope| {
            let world: &World = world;
            for system in self.systems.values_mut().filter_map(|container| {
                if container.active {
                    Some(&mut container.system)
                } else {
                    None
                }
            }) {
                scope.execute(move || system.run(world));
            }
            scope.join_all();
        });
        world.flush_mod_queues();
    }*/
}

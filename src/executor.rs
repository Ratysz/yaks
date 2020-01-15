use fixedbitset::FixedBitSet;
use fxhash::FxHasher64;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

use crate::{
    borrows::{ArchetypeSet, CondensedBorrows, SystemBorrows, TypeSet},
    error::NoSuchSystem,
    System, World,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex(usize);

struct SystemContainer {
    system: System,
    active: bool,
    borrows: SystemBorrows,
    condensed: CondensedBorrows,
    archetypes: ArchetypeSet,
}

impl SystemContainer {
    fn new(system: System) -> Self {
        let mut borrows = SystemBorrows::default();
        system.inner().write_borrows(&mut borrows);
        let archetypes = ArchetypeSet::default();
        Self {
            system,
            active: true,
            borrows,
            condensed: CondensedBorrows::with_capacity(0, 0),
            archetypes,
        }
    }
}

pub struct Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    systems: HashMap<SystemIndex, SystemContainer, BuildHasherDefault<FxHasher64>>,
    system_handles: HashMap<H, SystemIndex>,
    free_indices: Vec<SystemIndex>,
    dirty: bool,
    all_resources: TypeSet,
    all_components: TypeSet,
    current_resources: FixedBitSet,
    current_components: FixedBitSet,
}

impl<H> Default for Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self {
            systems: Default::default(),
            free_indices: Default::default(),
            system_handles: HashMap::default(),
            dirty: true,
            all_resources: Default::default(),
            all_components: Default::default(),
            current_resources: Default::default(),
            current_components: Default::default(),
        }
    }
}

impl<H> Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    pub fn new() -> Self {
        Default::default()
    }

    fn new_system_index(&mut self) -> SystemIndex {
        if let Some(index) = self.free_indices.pop() {
            index
        } else {
            SystemIndex(self.systems.len())
        }
    }

    fn add_inner(&mut self, handle: Option<H>, system: System) -> Option<System> {
        let container = SystemContainer::new(system);
        let index = match handle {
            Some(handle) => self
                .system_handles
                .get(&handle)
                .copied()
                .unwrap_or_else(|| {
                    let index = self.new_system_index();
                    self.system_handles.insert(handle, index);
                    index
                }),
            None => self.new_system_index(),
        };
        self.dirty = true;
        self.systems
            .insert(index, container)
            .map(|container| container.system)
    }

    fn maintain(&mut self) {
        if self.dirty {
            self.all_resources.clear();
            self.all_components.clear();
            for container in self.systems.values() {
                self.all_resources
                    .extend(&container.borrows.resources_mutable);
                self.all_resources
                    .extend(&container.borrows.resources_immutable);
                self.all_components
                    .extend(&container.borrows.components_mutable);
                self.all_components
                    .extend(&container.borrows.components_immutable);
            }
            for container in self.systems.values_mut() {
                container.condensed = container
                    .borrows
                    .condense(&self.all_resources, &self.all_components);
            }
            self.current_resources.grow(self.all_resources.len());
            self.current_components.grow(self.all_components.len());
            self.dirty = false;
        }
    }

    pub fn add(&mut self, system: System) {
        self.add_inner(None, system);
    }

    pub fn with(mut self, system: System) -> Self {
        self.add(system);
        self
    }

    pub fn add_with_handle(&mut self, handle: H, system: System) -> Option<System> {
        self.add_inner(Some(handle), system)
    }

    pub fn with_handle(mut self, handle: H, system: System) -> Self {
        self.add_with_handle(handle, system);
        self
    }

    pub fn remove(&mut self, handle: &H) -> Option<System> {
        self.system_handles
            .remove(handle)
            .and_then(|index| self.systems.remove(&index))
            .map(|container| container.system)
    }

    pub fn contains(&mut self, handle: &H) -> bool {
        self.system_handles.contains_key(handle)
    }

    pub fn get_mut(&mut self, handle: &H) -> Result<&mut System, NoSuchSystem> {
        let index = self.system_handles.get(handle).ok_or(NoSuchSystem)?;
        self.systems
            .get_mut(&index)
            .map(|container| &mut container.system)
            .ok_or(NoSuchSystem)
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

    pub fn run(&mut self, world: &mut World) {
        self.maintain();
        self.systems
            .values_mut()
            .filter(|container| container.active)
            .for_each(|container| container.system.run(world));
        world.flush_mod_queues();
    }

    #[cfg(feature = "parallel")]
    pub fn run_parallel(&mut self, world: &mut World) {
        self.maintain();
        unimplemented!();
    }
}

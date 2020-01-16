use fixedbitset::FixedBitSet;
use fxhash::FxHasher64;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

#[cfg(feature = "parallel")]
use crate::Threadpool;
use crate::{
    borrows::{ArchetypeSet, CondensedBorrows, SystemBorrows, TypeSet},
    error::NoSuchSystem,
    System, World,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex(usize);

struct SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    system: System,
    dependencies: Vec<H>,
    active: bool,
    borrows: SystemBorrows,
    condensed: CondensedBorrows,
    archetypes: ArchetypeSet,
}

impl<H> SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    fn new(system: System, dependencies: Vec<H>) -> Self {
        let mut borrows = SystemBorrows::default();
        system.inner().write_borrows(&mut borrows);
        let archetypes = ArchetypeSet::default();
        Self {
            system,
            dependencies,
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
    systems: HashMap<SystemIndex, SystemContainer<H>, BuildHasherDefault<FxHasher64>>,
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

    fn add_inner(
        &mut self,
        handle: Option<H>,
        dependencies: Vec<H>,
        system: System,
    ) -> Option<System> {
        let container = SystemContainer::new(system, dependencies);
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

    fn get_container(&self, handle: &H) -> Result<&SystemContainer<H>, NoSuchSystem> {
        let index = self.system_handles.get(handle).ok_or(NoSuchSystem)?;
        self.systems
            .get(&index)
            .ok_or_else(|| panic!("system handles should always map to valid system indices"))
    }

    fn get_mut_container(&mut self, handle: &H) -> Result<&mut SystemContainer<H>, NoSuchSystem> {
        let index = self.system_handles.get(handle).ok_or(NoSuchSystem)?;
        self.systems
            .get_mut(&index)
            .ok_or_else(|| panic!("system handles should always map to valid system indices"))
    }

    pub fn add<A>(&mut self, args: A) -> Option<System>
    where
        A: Into<SystemInsertionArguments<H>>,
    {
        let SystemInsertionArguments {
            system,
            handle,
            dependencies,
        } = args.into();
        self.add_inner(handle, dependencies, system)
    }

    pub fn with<A>(mut self, args: A) -> Self
    where
        A: Into<SystemInsertionArguments<H>>,
    {
        self.add(args);
        self
    }

    pub fn remove(&mut self, handle: &H) -> Option<System> {
        self.dirty = true;
        self.system_handles
            .remove(handle)
            .and_then(|index| self.systems.remove(&index))
            .map(|container| container.system)
    }

    pub fn contains(&mut self, handle: &H) -> bool {
        self.system_handles.contains_key(handle)
    }

    pub fn get_mut(&mut self, handle: &H) -> Result<&mut System, NoSuchSystem> {
        self.get_mut_container(handle)
            .map(|container| &mut container.system)
    }

    pub fn is_active(&self, handle: &H) -> Result<bool, NoSuchSystem> {
        self.get_container(handle).map(|container| container.active)
    }

    pub fn set_active(&mut self, handle: &H, active: bool) -> Result<(), NoSuchSystem> {
        self.get_mut_container(handle)
            .map(|container| container.active = active)
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
    pub fn run_parallel<P>(&mut self, world: &mut World, threadpool: P)
    where
        P: Threadpool,
    {
        self.maintain();
        self.current_resources.clear();
        self.current_components.clear();
        //for container in self.systems
    }
}

pub struct SystemInsertionArguments<H>
where
    H: Hash + Eq + PartialEq,
{
    handle: Option<H>,
    dependencies: Vec<H>,
    system: System,
}

impl<H> From<System> for SystemInsertionArguments<H>
where
    H: Hash + Eq + PartialEq,
{
    fn from(args: System) -> Self {
        SystemInsertionArguments {
            handle: None,
            dependencies: Vec::default(),
            system: args,
        }
    }
}

impl<H> From<(H, System)> for SystemInsertionArguments<H>
where
    H: Hash + Eq + PartialEq,
{
    fn from(args: (H, System)) -> Self {
        SystemInsertionArguments {
            handle: Some(args.0),
            dependencies: Vec::default(),
            system: args.1,
        }
    }
}

impl<H> From<(Vec<H>, System)> for SystemInsertionArguments<H>
where
    H: Hash + Eq + PartialEq,
{
    fn from(args: (Vec<H>, System)) -> Self {
        SystemInsertionArguments {
            handle: None,
            dependencies: args.0,
            system: args.1,
        }
    }
}

impl<'a, H> From<(H, Vec<H>, System)> for SystemInsertionArguments<H>
where
    H: Hash + Eq + PartialEq,
{
    fn from(args: (H, Vec<H>, System)) -> Self {
        SystemInsertionArguments {
            handle: Some(args.0),
            dependencies: args.1,
            system: args.2,
        }
    }
}

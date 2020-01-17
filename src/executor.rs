#[cfg(feature = "parallel")]
use crossbeam::{deque::Injector, sync::Parker};
use fxhash::FxHasher64;
use hecs::World;
use resources::Resources;
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hash},
};

#[cfg(feature = "parallel")]
use crate::Threadpool;
use crate::{
    borrows::{ArchetypeSet, CondensedBorrows, SystemBorrows, TypeSet},
    error::NoSuchSystem,
    ModQueuePool, System,
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

    systems_sorted: Vec<SystemIndex>,
    systems_to_run: Vec<SystemIndex>,
    current_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
    finished_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
}

impl<H> Default for Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self {
            systems: Default::default(),
            system_handles: Default::default(),
            free_indices: Default::default(),

            dirty: true,
            all_resources: Default::default(),
            all_components: Default::default(),

            systems_sorted: Default::default(),
            systems_to_run: Default::default(),
            current_systems: Default::default(),
            finished_systems: Default::default(),
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
            let mut sorted = Vec::new();
            sorted.extend(self.systems.keys());
            sorted.sort_by(|a, b| {
                let a_len = self
                    .systems
                    .get(a)
                    .expect("this key should be present at this point")
                    .dependencies
                    .len();
                let b_len = &self
                    .systems
                    .get(b)
                    .expect("this key should be present at this point")
                    .dependencies
                    .len();
                a_len.cmp(b_len)
            });
            self.systems_sorted.clear();
            self.systems_sorted.extend(sorted);
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

    pub fn run(&mut self, world: &World, resources: &Resources, mod_queues: &ModQueuePool) {
        self.maintain();
        self.systems
            .values_mut()
            .filter(|container| container.active)
            .for_each(|container| container.system.run(world, resources, mod_queues));
    }

    #[cfg(feature = "parallel")]
    pub fn run_parallel<P>(&mut self, _world: &mut World, _threadpool: &mut P)
    where
        P: Threadpool,
    {
        self.maintain();
        self.systems_to_run.clear();
        self.current_systems.clear();
        self.finished_systems.clear();
        for index in &self.systems_sorted {
            if self
                .systems
                .get(index)
                .expect("this key should be present at this point")
                .active
            {
                self.systems_to_run.push(*index);
            }
        }
        /*while !self.systems_to_run.is_empty() {
            for i in 0..self.systems_to_run.len() {
                if self.can_run_now(i) {
                    let container = &mut self
                        .systems
                        .get_mut(&self.systems_to_run[i])
                        .expect("this key should be present at this point");
                    let system = &mut container.system;
                    threadpool.execute(|| system.run(world));
                }
            }
        }*/
        //world.flush_mod_queues();
    }

    fn can_run_now(&self, system_to_run_index: usize) -> bool {
        let container = self
            .systems
            .get(&self.systems_to_run[system_to_run_index])
            .expect("this key should be present at this point");
        for dependency in &container.dependencies {
            if !self.finished_systems.contains(
                self.system_handles
                    .get(dependency)
                    .expect("system handles should always map to valid system indices"),
            ) {
                return false;
            }
        }
        for current in self.current_systems.iter().map(|index| {
            self.systems
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

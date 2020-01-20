use fxhash::FxHasher64;
use hecs::World;
use resources::Resources;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

use crate::{error::NoSuchSystem, system_container::SystemContainer, ModQueuePool, System};

#[cfg(feature = "parallel")]
use crossbeam::channel::{self, Receiver, Sender};
#[cfg(feature = "parallel")]
use std::collections::HashSet;

#[cfg(feature = "parallel")]
use crate::borrows::{BorrowsContainer, TypeSet};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct SystemIndex(usize);

pub struct Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    pub(crate) systems: HashMap<SystemIndex, SystemContainer<H>, BuildHasherDefault<FxHasher64>>,
    pub(crate) system_handles: HashMap<H, SystemIndex>,
    pub(crate) free_indices: Vec<SystemIndex>,
    pub(crate) systems_sorted: Vec<SystemIndex>,
    pub(crate) dirty: bool,

    #[cfg(feature = "parallel")]
    pub(crate) borrows: HashMap<SystemIndex, BorrowsContainer, BuildHasherDefault<FxHasher64>>,
    #[cfg(feature = "parallel")]
    pub(crate) all_resources: TypeSet,
    #[cfg(feature = "parallel")]
    pub(crate) all_components: TypeSet,
    #[cfg(feature = "parallel")]
    pub(crate) systems_to_run: Vec<SystemIndex>,
    #[cfg(feature = "parallel")]
    pub(crate) current_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
    #[cfg(feature = "parallel")]
    pub(crate) finished_systems: HashSet<SystemIndex, BuildHasherDefault<FxHasher64>>,
    #[cfg(feature = "parallel")]
    pub(crate) sender: Sender<SystemIndex>,
    #[cfg(feature = "parallel")]
    pub(crate) receiver: Receiver<SystemIndex>,
}

impl<H> Default for Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        #[cfg(feature = "parallel")]
        let (sender, receiver) = channel::bounded(1);
        Self {
            systems: Default::default(),
            system_handles: Default::default(),
            free_indices: Default::default(),
            systems_sorted: Default::default(),
            dirty: true,

            #[cfg(feature = "parallel")]
            borrows: Default::default(),
            #[cfg(feature = "parallel")]
            all_resources: Default::default(),
            #[cfg(feature = "parallel")]
            all_components: Default::default(),
            #[cfg(feature = "parallel")]
            systems_to_run: Default::default(),
            #[cfg(feature = "parallel")]
            current_systems: Default::default(),
            #[cfg(feature = "parallel")]
            finished_systems: Default::default(),
            #[cfg(feature = "parallel")]
            sender,
            #[cfg(feature = "parallel")]
            receiver,
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

    fn add_inner(
        &mut self,
        handle: Option<H>,
        dependencies: Vec<H>,
        system: System,
    ) -> Option<System> {
        #[cfg(feature = "parallel")]
        let borrows_container = BorrowsContainer::new(&system);
        let system_container = SystemContainer::new(system, dependencies);
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
        #[cfg(feature = "parallel")]
        self.borrows.insert(index, borrows_container);
        self.systems
            .insert(index, system_container)
            .map(|system_container| system_container.unwrap_system())
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
            .and_then(|index| {
                #[cfg(feature = "parallel")]
                self.borrows.remove(&index);
                self.systems.remove(&index)
            })
            .map(|system_container| system_container.unwrap_system())
    }

    pub fn contains(&mut self, handle: &H) -> bool {
        self.system_handles.contains_key(handle)
    }

    pub fn get_mut(
        &mut self,
        handle: &H,
    ) -> Result<impl std::ops::DerefMut<Target = System> + '_, NoSuchSystem> {
        self.get_mut_container(handle)
            .map(|system_container| system_container.system_mut())
    }

    pub fn is_active(&self, handle: &H) -> Result<bool, NoSuchSystem> {
        self.get_container(handle)
            .map(|system_container| system_container.active)
    }

    pub fn set_active(&mut self, handle: &H, active: bool) -> Result<(), NoSuchSystem> {
        self.get_mut_container(handle)
            .map(|system_container| system_container.active = active)
    }

    pub(crate) fn maintain(&mut self) {
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
        }); // TODO improve
        self.systems_sorted.clear();
        self.systems_sorted.extend(sorted);
        self.dirty = false;
    }

    pub fn run(&mut self, world: &World, resources: &Resources, mod_queues: &ModQueuePool) {
        if self.dirty {
            self.maintain();
        }
        self.systems
            .values_mut()
            .filter(|system_container| system_container.active)
            .for_each(|system_container| {
                system_container
                    .system_mut()
                    .run(world, resources, mod_queues)
            });
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

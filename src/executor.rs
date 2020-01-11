use fxhash::FxHasher64;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

use crate::{
    error::{NoSuchSystem, NonUniqueSystemHandle},
    system::{ArchetypeSet, SystemBorrows, TypeSet},
    System, World,
};

pub trait SystemHandle: Hash + Eq + PartialEq {}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex(usize);

struct SystemContainer {
    system: System,
    active: bool,
}

pub struct Executor<H = ()>
where
    H: Hash + Eq + PartialEq,
{
    systems: HashMap<SystemIndex, SystemContainer, BuildHasherDefault<FxHasher64>>,
    system_handles: HashMap<H, SystemIndex>,
    free_indices: Vec<SystemIndex>,
    //stages: Vec<Stage1>,
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
            //stages: Default::default(),
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

    fn add_inner(&mut self, system: System, active: bool) -> SystemIndex {
        let index = if let Some(index) = self.free_indices.pop() {
            index
        } else {
            SystemIndex(self.systems.len())
        };
        let container = SystemContainer { system, active };
        self.systems.insert(index, container);
        /*let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&container.system.borrows()))
        {
            Some((index, stage)) => (index, stage),
            None => {
                self.stages.push(Stage1::default());
                (
                    self.stages.len() - 1,
                    self.stages
                        .last_mut()
                        .expect("there has to be at least one stage at this point"),
                )
            }
        };*/
        index
    }

    pub fn add(&mut self, system: System) {
        self.add_inner(system, true);
    }

    pub fn with(mut self, system: System) -> Self {
        self.add(system);
        self
    }

    pub fn run(&mut self, world: &mut World) {
        let mut queues = self
            .systems
            .values_mut()
            .filter(|container| container.active)
            .map(|container| container.system.run_with_deferred_modification(world));
        let queue = queues.next().map(|queue| {
            queues.fold(queue, |mut first, current| {
                first.absorb(current);
                first
            })
        });
        if let Some(queue) = queue {
            world.apply_all(queue);
        }
    }

    #[allow(dead_code, unused_variables)]
    pub fn run_parallel(&mut self, world: &mut World) {
        unimplemented!()
        /*let mut queues = self
            .stages
            .iter_mut()
            .map(|stage| stage.run(world))
            .filter_map(|option| option);
        let queue = queues.next().map(|queue| {
            queues.fold(queue, |mut first, current| {
                first.absorb(current);
                first
            })
        });
        if let Some(queue) = queue {
            world.apply_all(queue);
        }*/
    }
}

impl<H> Executor<H>
where
    H: SystemHandle,
{
    #[allow(clippy::map_entry)]
    fn add_with_handle_inner(
        &mut self,
        handle: H,
        system: System,
        active: bool,
    ) -> Result<(), NonUniqueSystemHandle> {
        if self.system_handles.contains_key(&handle) {
            Err(NonUniqueSystemHandle)
        } else {
            let index = self.add_inner(system, active);
            self.system_handles.insert(handle, index);
            Ok(())
        }
    }

    pub fn add_with_handle(
        &mut self,
        handle: H,
        system: System,
    ) -> Result<(), NonUniqueSystemHandle> {
        self.add_with_handle_inner(handle, system, true)
    }

    pub fn add_with_handle_deactivated(
        &mut self,
        handle: H,
        system: System,
    ) -> Result<(), NonUniqueSystemHandle> {
        self.add_with_handle_inner(handle, system, false)
    }

    pub fn with_handle(mut self, handle: H, system: System) -> Self {
        if let Err(error) = self.add_with_handle(handle, system) {
            panic!("{}", error);
        }
        self
    }

    pub fn with_handle_deactivated(mut self, handle: H, system: System) -> Self {
        if let Err(error) = self.add_with_handle_deactivated(handle, system) {
            panic!("{}", error);
        }
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
}

/*#[derive(Default)]
struct Stage1 {
    stages: Vec<Stage2>,
    resources_immutable: TypeSet,
    resources_mutable: TypeSet,
}

impl Stage1 {
    fn is_compatible(&self, borrows: &SystemBorrows) -> bool {
        borrows
            .resources_mutable
            .is_disjoint(&self.resources_mutable)
            && borrows
                .resources_immutable
                .is_disjoint(&self.resources_mutable)
            && borrows
                .resources_mutable
                .is_disjoint(&self.resources_immutable)
    }

    fn add(&mut self, container: SystemContainer) -> SystemIndex {
        self.resources_immutable
            .extend(&container.system.borrows().resources_immutable);
        self.resources_mutable
            .extend(&container.system.borrows().resources_mutable);
        let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&container.system.borrows()))
        {
            Some((index, stage)) => (index, stage),
            None => {
                self.stages.push(Stage2::default());
                (
                    self.stages.len() - 1,
                    self.stages
                        .last_mut()
                        .expect("there has to be at least one stage at this point"),
                )
            }
        };
        let mut system_index = stage.add(container);
        system_index.stage2 = index;
        system_index
    }

    fn is_active(&self, system_index: SystemIndex) -> bool {
        self.stages[system_index.stage2].is_active(system_index.index)
    }

    fn set_active(&mut self, system_index: SystemIndex, active: bool) {
        self.stages[system_index.stage2].set_active(system_index.index, active);
    }

    fn run(&mut self, world: &World) -> Option<ModificationQueue> {
        let mut queues = self
            .stages
            .iter_mut()
            .map(|stage| stage.run(world))
            .filter_map(|option| option);
        queues.next().map(|queue| {
            queues.fold(queue, |mut first, current| {
                first.absorb(current);
                first
            })
        })
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

#[derive(Default)]
struct Stage2 {
    systems: Vec<SystemContainer>,
    components_immutable: TypeSet,
    components_mutable: TypeSet,
}

impl Stage2 {
    fn is_compatible(&self, borrows: &SystemBorrows) -> bool {
        borrows
            .components_mutable
            .is_disjoint(&self.components_mutable)
            && borrows
                .components_immutable
                .is_disjoint(&self.components_mutable)
            && borrows
                .components_mutable
                .is_disjoint(&self.components_immutable)
    }

    fn add(&mut self, container: SystemContainer) -> SystemIndex {
        self.components_immutable
            .extend(&container.system.borrows().components_immutable);
        self.components_mutable
            .extend(&container.system.borrows().components_mutable);
        self.systems.push(container);
        SystemIndex {
            stage1: 0,
            stage2: 0,
            index: self.systems.len() - 1,
        }
    }

    fn is_active(&self, index: usize) -> bool {
        self.systems[index].active
    }

    fn set_active(&mut self, index: usize, active: bool) {
        self.systems[index].active = active;
    }

    fn run(&mut self, world: &World) -> Option<ModificationQueue> {
        let mut queues = self
            .systems
            .iter_mut()
            .filter(|container| container.active)
            .map(|container| container.system.run_without_updating_archetypes(world));
        queues.next().map(|queue| {
            queues.fold(queue, |mut first, current| {
                first.absorb(current);
                first
            })
        })
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}*/

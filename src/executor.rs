use std::{collections::HashMap, hash::Hash};

use crate::{
    system::{ArchetypeSet, SystemBorrows, TypeSet},
    ModificationQueue, NoSuchSystem, NonUniqueSystemHandle, System, World,
};

pub trait SystemHandle: Hash + Eq + PartialEq {}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex {
    stage1: usize,
    stage2: usize,
    index: usize,
}

struct SystemContainer {
    system: System,
    active: bool,
}

pub struct Executor<H = ()>
where
    H: Hash + Eq + PartialEq,
{
    stages: Vec<Stage1>,
    system_map: HashMap<H, SystemIndex>,
}

impl<H> Default for Executor<H>
where
    H: Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self {
            stages: Default::default(),
            system_map: HashMap::with_hasher(Default::default()),
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
        let container = SystemContainer { system, active };
        let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&container.system.borrows))
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
        };
        let mut system_index = stage.add(container);
        system_index.stage1 = index;
        system_index
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
            .stages
            .iter_mut()
            .map(|stage| stage.run(world))
            .filter_map(|option| option);
        let queue = queues.next().map(|queue| {
            queues.fold(queue, |mut first, current| {
                first.merge(current);
                first
            })
        });
        if let Some(queue) = queue {
            queue.apply_all(world);
        }
    }

    #[allow(dead_code, unused_variables)]
    pub fn run_parallel(&mut self, world: &mut World) {
        unimplemented!()
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
        if self.system_map.contains_key(&handle) {
            Err(NonUniqueSystemHandle)
        } else {
            let index = self.add_inner(system, active);
            self.system_map.insert(handle, index);
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
        self.system_map
            .get(handle)
            .ok_or_else(|| NoSuchSystem)
            .map(|system_index| self.stages[system_index.stage1].is_active(*system_index))
    }

    pub fn set_active(&mut self, handle: &H, active: bool) -> Result<(), NoSuchSystem> {
        match self.system_map.get(handle) {
            Some(system_index) => {
                self.stages[system_index.stage1].set_active(*system_index, active);
                Ok(())
            }
            None => Err(NoSuchSystem),
        }
    }
}

#[derive(Default)]
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
            .extend(&container.system.borrows.resources_immutable);
        self.resources_mutable
            .extend(&container.system.borrows.resources_mutable);
        let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&container.system.borrows))
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
                first.merge(current);
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
            .extend(&container.system.borrows.components_immutable);
        self.components_mutable
            .extend(&container.system.borrows.components_mutable);
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
                first.merge(current);
                first
            })
        })
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

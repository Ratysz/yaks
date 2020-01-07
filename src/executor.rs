use std::{collections::HashMap, hash::Hash};

use crate::{
    borrows::{SystemWithBorrows, TypeSet},
    NoSuchSystem, NonUniqueSystemHandle, System, World,
};

pub trait SystemHandle: Hash + Eq + PartialEq {}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct SystemIndex {
    stage1: usize,
    stage2: usize,
    index: usize,
}

pub struct Executor<H>
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

    fn add_common(&mut self, system: Box<dyn System>, active: bool) -> SystemIndex {
        let swb = SystemWithBorrows::new(system);
        let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&swb))
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
        let mut system_index = stage.add(swb, active);
        system_index.stage1 = index;
        system_index
    }

    pub fn run(&mut self, world: &mut World) {
        self.stages.iter_mut().for_each(|stage| stage.run(world));
    }

    #[allow(dead_code, unused_variables)]
    pub fn run_parallel(&mut self, world: &mut World) {
        unimplemented!()
    }
}

impl Executor<()> {
    pub fn add(&mut self, system: Box<dyn System>) {
        self.add_common(system, true);
    }

    pub fn with(mut self, system: Box<dyn System>) -> Self {
        self.add(system);
        self
    }
}

impl<H> Executor<H>
where
    H: SystemHandle,
{
    #[allow(clippy::map_entry)]
    fn add_with_handle(
        &mut self,
        handle: H,
        system: Box<dyn System>,
        active: bool,
    ) -> Result<(), NonUniqueSystemHandle> {
        if self.system_map.contains_key(&handle) {
            Err(NonUniqueSystemHandle)
        } else {
            let index = self.add_common(system, active);
            self.system_map.insert(handle, index);
            Ok(())
        }
    }

    pub fn add(&mut self, handle: H, system: Box<dyn System>) -> Result<(), NonUniqueSystemHandle> {
        self.add_with_handle(handle, system, true)
    }

    pub fn add_inactive(
        &mut self,
        handle: H,
        system: Box<dyn System>,
    ) -> Result<(), NonUniqueSystemHandle> {
        self.add_with_handle(handle, system, false)
    }

    pub fn with(mut self, handle: H, system: Box<dyn System>) -> Self {
        if let Err(error) = self.add(handle, system) {
            panic!("{}", error);
        }
        self
    }

    pub fn with_inactive(mut self, handle: H, system: Box<dyn System>) -> Self {
        if let Err(error) = self.add_inactive(handle, system) {
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
    fn is_compatible(&self, swb: &SystemWithBorrows) -> bool {
        swb.borrows
            .are_resource_borrows_compatible(&self.resources_immutable, &self.resources_mutable)
    }

    fn add(&mut self, swb: SystemWithBorrows, active: bool) -> SystemIndex {
        self.resources_immutable
            .extend(&swb.borrows.resources_immutable);
        self.resources_mutable
            .extend(&swb.borrows.resources_mutable);
        let (index, stage) = match self
            .stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.is_compatible(&swb))
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
        let mut system_index = stage.add(swb, active);
        system_index.stage2 = index;
        system_index
    }

    fn is_active(&self, system_index: SystemIndex) -> bool {
        self.stages[system_index.stage2].is_active(system_index.index)
    }

    fn set_active(&mut self, system_index: SystemIndex, active: bool) {
        self.stages[system_index.stage2].set_active(system_index.index, active);
    }

    fn run(&mut self, world: &World) {
        self.stages.iter_mut().for_each(|stage| stage.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

#[derive(Default)]
struct Stage2 {
    systems: Vec<(bool, Box<dyn System>)>,
    components_immutable: TypeSet,
    components_mutable: TypeSet,
}

impl Stage2 {
    fn is_compatible(&self, swb: &SystemWithBorrows) -> bool {
        swb.borrows
            .are_component_borrows_compatible(&self.components_immutable, &self.components_mutable)
    }

    fn add(&mut self, swb: SystemWithBorrows, active: bool) -> SystemIndex {
        self.components_immutable
            .extend(&swb.borrows.components_immutable);
        self.components_mutable
            .extend(&swb.borrows.components_mutable);
        self.systems.push((active, swb.system));
        SystemIndex {
            stage1: 0,
            stage2: 0,
            index: self.systems.len() - 1,
        }
    }

    fn is_active(&self, index: usize) -> bool {
        self.systems[index].0
    }

    fn set_active(&mut self, index: usize, active: bool) {
        self.systems[index].0 = active;
    }

    fn run(&mut self, world: &World) {
        self.systems
            .iter_mut()
            .filter(|(active, _)| *active)
            .for_each(|(_, system)| system.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

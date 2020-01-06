use std::vec::Drain;

use crate::{
    metadata::{ArchetypeSet, SystemWithMetadata, TypeSet},
    System, World,
};

#[derive(Default)]
pub struct Executor {
    stages: Vec<Stage1>,
}

impl Executor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, system: Box<dyn System>) {
        let swm = SystemWithMetadata::new(system);
        if let Some(stage) = self
            .stages
            .iter_mut()
            .find(|stage| stage.is_compatible(&swm))
        {
            stage.add(swm)
        }
    }

    pub fn with(mut self, system: Box<dyn System>) -> Self {
        self.add(system);
        self
    }

    pub fn run(&mut self, world: &mut World) {
        self.stages.iter_mut().for_each(|stage| stage.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &mut World) {
        unimplemented!()
    }
}

#[derive(Default)]
struct Stage1 {
    stages: Vec<Stage2>,
    resources_immutable: TypeSet,
    resources_mutable: TypeSet,

    unassigned: Vec<SystemWithMetadata>,
    unassigned_tail: Vec<SystemWithMetadata>,
    archetypes: ArchetypeSet,
}

impl Stage1 {
    fn is_compatible(&self, swm: &SystemWithMetadata) -> bool {
        swm.metadata
            .are_resource_borrows_compatible(&self.resources_immutable, &self.resources_mutable)
    }

    fn add(&mut self, swm: SystemWithMetadata) {
        self.resources_immutable
            .extend(&swm.metadata.resources_immutable);
        self.resources_mutable
            .extend(&swm.metadata.resources_mutable);
        self.unassigned_tail.push(swm);
    }

    fn rebuild(&mut self, world: &World) {
        for stage in &mut self.stages {
            self.unassigned.extend(stage.drain());
        }
        self.unassigned.extend(self.unassigned_tail.drain(..));

        for swm in self.unassigned.drain(..) {
            swm.system
                .write_touched_archetypes(world, &mut self.archetypes);
            let archetypes = &self.archetypes;
            let stage = match self
                .stages
                .iter_mut()
                .find(|stage| stage.is_compatible(&swm, archetypes))
            {
                Some(stage) => stage,
                None => {
                    self.stages.push(Stage2::default());
                    self.stages.last_mut().unwrap()
                }
            };
            stage.add(swm, &mut self.archetypes);
        }
    }

    fn run(&mut self, world: &World) {
        self.rebuild(world);
        self.stages.iter_mut().for_each(|stage| stage.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

#[derive(Default)]
struct Stage2 {
    systems: Vec<SystemWithMetadata>,
    components_immutable: TypeSet,
    components_mutable: TypeSet,
    archetypes: ArchetypeSet,
}

impl Stage2 {
    fn is_compatible(&self, swm: &SystemWithMetadata, archetypes: &ArchetypeSet) -> bool {
        self.archetypes.is_disjoint(archetypes)
            || (swm.metadata.are_component_borrows_compatible(
                &self.components_immutable,
                &self.components_mutable,
            ))
    }

    fn add(&mut self, swm: SystemWithMetadata, archetypes: &mut ArchetypeSet) {
        self.archetypes.extend(archetypes.drain());
        self.components_immutable
            .extend(&swm.metadata.components_immutable);
        self.components_mutable
            .extend(&swm.metadata.components_mutable);
        self.systems.push(swm);
    }

    fn drain(&mut self) -> Drain<'_, SystemWithMetadata> {
        self.archetypes.clear();
        self.components_immutable.clear();
        self.components_mutable.clear();
        self.systems.drain(..)
    }

    fn run(&mut self, world: &World) {
        self.systems
            .iter_mut()
            .for_each(|swm| swm.system.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Executor, SystemBuilder};

    struct Resource1;

    struct Resource2;

    struct Resource3;

    struct Component1;

    struct Component2;

    struct Component3;

    struct Component4;

    #[test]
    fn basic() {
        let mut executor = Executor::new();
        executor.add(SystemBuilder::<(), ()>::build(|_, _, _| ()));
    }
}

use std::vec::Drain;

use crate::{
    system::{ArchetypeSet, TypeSet},
    System, World,
};

struct SystemWithMetadata {
    system: Box<dyn System>,
    metadata: crate::SystemMetadata,
}

impl SystemWithMetadata {
    fn new(system: Box<dyn System>) -> Self {
        let mut metadata = Default::default();
        system.write_metadata(&mut metadata);
        Self { system, metadata }
    }
}

#[derive(Default)]
pub struct Executor {
    stages: Vec<Stage1>,
}

impl Executor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        let swm = SystemWithMetadata::new(system);
        if let Some(stage) = self
            .stages
            .iter_mut()
            .find(|stage| stage.is_compatible(&swm))
        {
            stage.add_system(swm)
        }
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
    resources: TypeSet,
    resources_mut: TypeSet,

    unassigned: Vec<SystemWithMetadata>,
    unassigned_tail: Vec<SystemWithMetadata>,
    archetypes: ArchetypeSet,
}

impl Stage1 {
    fn is_compatible(&self, swm: &SystemWithMetadata) -> bool {
        self.resources_mut.is_disjoint(&swm.metadata.resources_mut)
            && self.resources.is_disjoint(&swm.metadata.resources_mut)
            && self.resources_mut.is_disjoint(&swm.metadata.resources)
    }

    fn add_system(&mut self, swm: SystemWithMetadata) {
        self.resources.extend(&swm.metadata.resources);
        self.resources_mut.extend(&swm.metadata.resources_mut);
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
            stage.add_system(swm, &mut self.archetypes);
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
    components: TypeSet,
    components_mut: TypeSet,
    archetypes: ArchetypeSet,
}

impl Stage2 {
    fn is_compatible(&self, swm: &SystemWithMetadata, archetypes: &ArchetypeSet) -> bool {
        self.archetypes.is_disjoint(archetypes)
            || (self
                .components_mut
                .is_disjoint(&swm.metadata.components_mut)
                && self.components.is_disjoint(&swm.metadata.components_mut)
                && self.components_mut.is_disjoint(&swm.metadata.components))
    }

    fn add_system(&mut self, swm: SystemWithMetadata, archetypes: &mut ArchetypeSet) {
        self.archetypes.extend(archetypes.drain());
        self.components.extend(&swm.metadata.components);
        self.components_mut.extend(&swm.metadata.components_mut);
        self.systems.push(swm);
    }

    fn drain(&mut self) -> Drain<'_, SystemWithMetadata> {
        self.archetypes.clear();
        self.components.clear();
        self.components_mut.clear();
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
        executor.add_system(SystemBuilder::<(), ()>::build(|_, _, _| ()));
    }
}

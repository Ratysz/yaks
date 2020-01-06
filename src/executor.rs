use crate::{
    metadata::{SystemWithMetadata, TypeSet},
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
        if let Some(stage) = self
            .stages
            .iter_mut()
            .find(|stage| stage.is_compatible(&swm))
        {
            stage.add(swm)
        }
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
    systems: Vec<Box<dyn System>>,
    components_immutable: TypeSet,
    components_mutable: TypeSet,
}

impl Stage2 {
    fn is_compatible(&self, swm: &SystemWithMetadata) -> bool {
        swm.metadata
            .are_component_borrows_compatible(&self.components_immutable, &self.components_mutable)
    }

    fn add(&mut self, swm: SystemWithMetadata) {
        self.components_immutable
            .extend(&swm.metadata.components_immutable);
        self.components_mutable
            .extend(&swm.metadata.components_mutable);
        self.systems.push(swm.system);
    }

    fn run(&mut self, world: &World) {
        self.systems.iter_mut().for_each(|system| system.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

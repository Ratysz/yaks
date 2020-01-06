use crate::{
    borrows::{SystemWithBorrows, TypeSet},
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
        let swb = SystemWithBorrows::new(system);
        if let Some(stage) = self
            .stages
            .iter_mut()
            .find(|stage| stage.is_compatible(&swb))
        {
            stage.add(swb)
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
    fn is_compatible(&self, swb: &SystemWithBorrows) -> bool {
        swb.borrows
            .are_resource_borrows_compatible(&self.resources_immutable, &self.resources_mutable)
    }

    fn add(&mut self, swb: SystemWithBorrows) {
        self.resources_immutable
            .extend(&swb.borrows.resources_immutable);
        self.resources_mutable
            .extend(&swb.borrows.resources_mutable);
        if let Some(stage) = self
            .stages
            .iter_mut()
            .find(|stage| stage.is_compatible(&swb))
        {
            stage.add(swb)
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
    fn is_compatible(&self, swb: &SystemWithBorrows) -> bool {
        swb.borrows
            .are_component_borrows_compatible(&self.components_immutable, &self.components_mutable)
    }

    fn add(&mut self, swb: SystemWithBorrows) {
        self.components_immutable
            .extend(&swb.borrows.components_immutable);
        self.components_mutable
            .extend(&swb.borrows.components_mutable);
        self.systems.push(swb.system);
    }

    fn run(&mut self, world: &World) {
        self.systems.iter_mut().for_each(|system| system.run(world));
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Executor, SystemBuilder, World};

    struct Resource1;

    struct Resource2;

    struct Resource3;

    struct Component1;

    struct Component2;

    struct Component3;

    struct Component4;

    #[test]
    fn basic() {
        let mut world = World::new();
        world.add_resource(0u32);
        let mut executor = Executor::new().with(SystemBuilder::<&mut u32, ()>::build(
            move |_, mut resource, _| {
                *resource += 1;
            },
        ));
        executor.run(&mut world);
        assert_eq!(*world.fetch::<&u32>(), 1);
    }
}

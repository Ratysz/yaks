use crate::{ExecutorBuilder, ResourceTuple, SystemClosure, SystemId};

pub struct ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    systems: Vec<(SystemId, Box<SystemClosure<'closures, Resources::Wrapped>>)>,
}

impl<'closures, Resources> ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        let ExecutorBuilder { mut systems, .. } = builder;
        let mut systems: Vec<_> = systems
            .drain()
            .map(|(id, system)| (id, system.closure))
            .collect();
        systems.sort_by(|(a, _), (b, _)| a.cmp(b));
        ExecutorSequential { systems }
    }

    pub fn force_archetype_recalculation(&mut self) {}

    pub fn run(&mut self, world: &hecs::World, wrapped: Resources::Wrapped) {
        for (_, closure) in &mut self.systems {
            closure(world, &wrapped);
        }
    }
}

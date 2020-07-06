use hecs::World;

use super::SystemClosure;
use crate::{ExecutorBuilder, ResourceTuple, SystemContext, SystemId};

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

    pub fn run(&mut self, world: &World, wrapped: Resources::Wrapped) {
        for (id, closure) in &mut self.systems {
            closure(
                SystemContext {
                    system_id: Some(*id),
                    world,
                },
                &wrapped,
            );
        }
    }
}

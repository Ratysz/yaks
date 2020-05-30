use hecs::World;

use crate::{ExecutorBuilder, ResourceTuple, ResourceWrap, SystemContext, WrappedResources};

#[cfg(feature = "parallel")]
use crate::ExecutorParallel;

#[cfg(not(feature = "parallel"))]
use crate::SystemId;

pub type SystemClosure<'closure, Cells> =
    dyn FnMut(SystemContext, &WrappedResources<Cells>) + Send + Sync + 'closure;

pub struct Executor<'closures, Resources>
where
    Resources: ResourceTuple,
{
    #[cfg(feature = "parallel")]
    inner: ExecutorParallel<'closures, Resources>,
    #[cfg(not(feature = "parallel"))]
    inner: ExecutorSequential<'closures, Resources>,
}

impl<'closures, Resources> Executor<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn builder() -> ExecutorBuilder<'closures, Resources> {
        ExecutorBuilder::new()
    }

    pub(crate) fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        Self {
            #[cfg(feature = "parallel")]
            inner: ExecutorParallel::build(builder),
            #[cfg(not(feature = "parallel"))]
            inner: ExecutorSequential::build(builder),
        }
    }

    pub fn force_archetype_recalculation(&mut self) {
        self.inner.force_archetype_recalculation();
    }

    pub fn run<ResourceTuple>(&mut self, world: &World, resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        self.inner.run(world, resources);
    }
}

#[cfg(not(feature = "parallel"))]
struct ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    borrows: Resources::Borrows,
    systems: Vec<(SystemId, Box<SystemClosure<'closures, Resources::Cells>>)>,
}

#[cfg(not(feature = "parallel"))]
impl<'closures, Resources> ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        let ExecutorBuilder { mut systems, .. } = builder;
        let mut systems: Vec<_> = systems
            .drain()
            .map(|(id, system)| (id, system.closure))
            .collect();
        systems.sort_by(|(a, _), (b, _)| a.cmp(b));
        ExecutorSequential {
            borrows: Resources::instantiate_borrows(),
            systems,
        }
    }

    fn force_archetype_recalculation(&mut self) {}

    fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        let wrapped = resources.wrap(&mut self.borrows);
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

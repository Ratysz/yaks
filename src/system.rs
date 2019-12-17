use fxhash::FxHasher64;
use std::{
    any::TypeId, collections::HashSet, hash::BuildHasherDefault, iter::FromIterator,
    marker::PhantomData,
};

use crate::{
    query_bundle::QueryBundle, resource_bundle::ResourceBundle, world::ArchetypeSet, World,
};

pub(crate) type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;

pub trait StaticSystem<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World);

    fn borrowed_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_components())
    }

    fn borrowed_mut_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_mut_components())
    }

    fn touched_archetypes(&self, world: &World) -> ArchetypeSet {
        Q::touched_archetypes(world)
    }
}

impl<R, Q, F> StaticSystem<R, Q> for F
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, R::Effectors, Q::Effectors),
{
    fn run(&mut self, world: &World) {
        self(world, R::effectors(), Q::effectors())
    }
}

pub struct StaticSystemBuilder<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> StaticSystemBuilder<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    pub fn build(
        system: impl FnMut(&World, R::Effectors, Q::Effectors),
    ) -> impl StaticSystem<R, Q> {
        system
    }
}

struct SystemBox<R, Q, F>
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, R::Effectors, Q::Effectors),
{
    phantom_data: PhantomData<(R, Q)>,
    system: F,
}

impl<R, Q, F> From<F> for SystemBox<R, Q, F>
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, R::Effectors, Q::Effectors),
{
    fn from(system: F) -> Self {
        Self {
            phantom_data: PhantomData,
            system,
        }
    }
}

pub trait DynamicSystem {
    fn run(&mut self, world: &World);

    fn borrowed_components(&self) -> TypeSet;

    fn borrowed_mut_components(&self) -> TypeSet;

    fn touched_archetypes(&self, world: &World) -> ArchetypeSet;
}

impl<R, Q, F> DynamicSystem for SystemBox<R, Q, F>
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, R::Effectors, Q::Effectors),
{
    fn run(&mut self, world: &World) {
        (self.system)(world, R::effectors(), Q::effectors())
    }

    fn borrowed_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_components())
    }

    fn borrowed_mut_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_mut_components())
    }

    fn touched_archetypes(&self, world: &World) -> ArchetypeSet {
        Q::touched_archetypes(world)
    }
}

pub struct DynamicSystemBuilder<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> DynamicSystemBuilder<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    pub fn build(
        system: impl FnMut(&World, R::Effectors, Q::Effectors) + 'static,
    ) -> Box<dyn DynamicSystem> {
        Box::new(SystemBox::<R, Q, _>::from(Box::new(system)))
    }
}

#[test]
fn test() {
    use crate::Fetch;
    let mut world = World::new();
    world.add_resource::<usize>(1);
    world.add_resource::<f32>(1.0);
    let mut system1 = StaticSystemBuilder::<(&usize, &mut f32), (&usize, Option<&usize>)>::build(
        |world, fetch, query| {
            let (res1, mut res2) = fetch.fetch(world);
            *res2 += 1.0;
        },
    );
    let mut system2 = DynamicSystemBuilder::<(&usize, &mut f32), (&usize, Option<&usize>)>::build(
        |world, fetch, query| {
            let (res1, mut res2) = fetch.fetch(world);
            *res2 += 1.0;
        },
    );
    system1.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system2.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

use fxhash::FxHasher64;
use std::{
    any::TypeId, collections::HashSet, hash::BuildHasherDefault, iter::FromIterator,
    marker::PhantomData,
};

use crate::{
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    world::ArchetypeSet,
    World,
};

pub(crate) type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;

pub trait System<'a, R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &'a World);

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

impl<'a, R, Q, F> System<'a, R, Q> for F
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&'a World, <R::Refs as Fetch<'a>>::Item, Q::Effectors),
{
    fn run(&mut self, world: &'a World) {
        self(world, R::fetch(world), Q::effectors())
    }
}

struct SystemBox<R, Q, F>
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, <R::Refs as Fetch>::Item, Q::Effectors),
{
    phantom_data: PhantomData<(R, Q)>,
    system: F,
}

impl<R, Q, F> From<F> for SystemBox<R, Q, F>
where
    R: ResourceBundle,
    Q: QueryBundle,
    F: FnMut(&World, <R::Refs as Fetch>::Item, Q::Effectors),
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
    F: FnMut(&World, <R::Refs as Fetch>::Item, Q::Effectors),
{
    fn run(&mut self, world: &World) {
        //self.system.run(world)
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

pub struct SystemBuilderStatic<'a, R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(&'a (), R, Q)>,
}

impl<'a, R, Q> SystemBuilderStatic<'a, R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    pub fn build(
        system: impl FnMut(&'a World, <R::Refs as Fetch<'a>>::Item, Q::Effectors),
    ) -> impl System<'a, R, Q> {
        system
    }
}

pub struct SystemBuilderDynamic<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> SystemBuilderDynamic<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    pub fn build(
        system: impl FnMut(&World, <R::Refs as Fetch>::Item, Q::Effectors) + 'static,
    ) -> Box<dyn DynamicSystem> {
        Box::new(SystemBox::<R, Q, _>::from(Box::new(system)))
    }
}

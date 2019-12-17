use std::{any::TypeId, marker::PhantomData};

use crate::{system::TypeSet, world::ArchetypeSet, Component, Query, QueryBorrow, World};

trait ElidedQuery {}

pub struct QueryEffector<Q>
where
    Q: Query + Send + Sync,
{
    phantom_data: PhantomData<Q>,
}

impl<Q> QueryEffector<Q>
where
    Q: Query + Send + Sync,
{
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }

    pub fn query<'a>(&self, world: &'a World) -> QueryBorrow<'a, Q> {
        world.query()
    }
}

pub trait QueryBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    fn borrowed_components() -> TypeSet;

    fn borrowed_mut_components() -> TypeSet;

    fn touched_archetypes(world: &World) -> ArchetypeSet;
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    fn borrowed_components() -> TypeSet {
        TypeSet::default()
    }

    fn borrowed_mut_components() -> TypeSet {
        TypeSet::default()
    }

    fn touched_archetypes(_: &World) -> ArchetypeSet {
        ArchetypeSet::default()
    }
}

impl<C> QueryBundle for &'_ C
where
    C: Component,
{
    type Effectors = QueryEffector<Self>;

    fn effectors() -> Self::Effectors {
        QueryEffector::new()
    }

    fn borrowed_components() -> TypeSet {
        let mut set = TypeSet::default();
        set.insert(TypeId::of::<C>());
        set
    }

    fn borrowed_mut_components() -> TypeSet {
        TypeSet::default()
    }

    fn touched_archetypes(world: &World) -> ArchetypeSet {
        world.touched_archetypes::<Self>()
    }
}

impl<C> QueryBundle for &'_ mut C
where
    C: Component,
{
    type Effectors = QueryEffector<Self>;

    fn effectors() -> Self::Effectors {
        QueryEffector::new()
    }

    fn borrowed_components() -> TypeSet {
        TypeSet::default()
    }

    fn borrowed_mut_components() -> TypeSet {
        let mut set = TypeSet::default();
        set.insert(TypeId::of::<C>());
        set
    }

    fn touched_archetypes(world: &World) -> ArchetypeSet {
        world.touched_archetypes::<Self>()
    }
}

impl<Q> QueryBundle for Option<Q>
where
    Q: Query + QueryBundle,
{
    type Effectors = QueryEffector<Self>;

    fn effectors() -> Self::Effectors {
        QueryEffector::new()
    }

    fn borrowed_components() -> TypeSet {
        Q::borrowed_components()
    }

    fn borrowed_mut_components() -> TypeSet {
        Q::borrowed_mut_components()
    }

    fn touched_archetypes(world: &World) -> ArchetypeSet {
        Q::touched_archetypes(world)
    }
}

use std::{any::TypeId, marker::PhantomData};

use crate::{system::ArchetypeSet, Component, Query, QueryBorrow, SystemMetadata, World};

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

pub trait QuerySingle: Send + Sync {
    type Effector;

    fn effector() -> Self::Effector;

    fn write_metadata(metadata: &mut SystemMetadata);

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet);
}

impl<C> QuerySingle for &'_ C
where
    C: Component,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    fn write_metadata(metadata: &mut SystemMetadata) {
        metadata.components.insert(TypeId::of::<C>());
    }

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes::<Self>(set);
    }
}

impl<C> QuerySingle for &'_ mut C
where
    C: Component,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    fn write_metadata(metadata: &mut SystemMetadata) {
        metadata.components_mut.insert(TypeId::of::<C>());
    }

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes::<Self>(set);
    }
}

impl<Q> QuerySingle for Option<Q>
where
    Q: QuerySingle,
    Self: Query,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    fn write_metadata(metadata: &mut SystemMetadata) {
        Q::write_metadata(metadata);
    }

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes::<Self>(set);
    }
}

pub trait QueryBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    fn write_metadata(metadata: &mut SystemMetadata);

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet);
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    fn write_metadata(_: &mut SystemMetadata) {}

    fn write_touched_archetypes(_: &World, _: &mut ArchetypeSet) {}
}

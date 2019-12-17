use std::{any::TypeId, marker::PhantomData};

use crate::{ArchetypeSet, Component, Query, QueryBorrow, TypeSet, World};

// TODO: decide if (&C1, &C2) is to be interpreted as Query<(&C1, &C2)> or (Query<&C1>, Query<&C2>).

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

    fn write_borrowed_components(set: &mut TypeSet);

    fn write_borrowed_mut_components(set: &mut TypeSet);

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet);
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    fn write_borrowed_components(_: &mut TypeSet) {}

    fn write_borrowed_mut_components(_: &mut TypeSet) {}

    fn write_touched_archetypes(_: &World, _: &mut ArchetypeSet) {}
}

impl<C> QueryBundle for &'_ C
where
    C: Component,
{
    type Effectors = QueryEffector<Self>;

    fn effectors() -> Self::Effectors {
        QueryEffector::new()
    }

    fn write_borrowed_components(set: &mut TypeSet) {
        set.insert(TypeId::of::<C>());
    }

    fn write_borrowed_mut_components(_: &mut TypeSet) {}

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes_for_query::<Self>(set);
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

    fn write_borrowed_components(_: &mut TypeSet) {}

    fn write_borrowed_mut_components(set: &mut TypeSet) {
        set.insert(TypeId::of::<C>());
    }

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes_for_query::<Self>(set);
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

    fn write_borrowed_components(set: &mut TypeSet) {
        Q::write_borrowed_components(set);
    }

    fn write_borrowed_mut_components(set: &mut TypeSet) {
        Q::write_borrowed_mut_components(set);
    }

    fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
        world.write_touched_archetypes_for_query::<Q>(set);
    }
}

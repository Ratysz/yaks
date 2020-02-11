use hecs::{Component, Query, QueryBorrow, World};
use std::marker::PhantomData;

#[cfg(feature = "parallel")]
use hecs::Access;
#[cfg(feature = "parallel")]
use std::any::TypeId;

#[cfg(feature = "parallel")]
use crate::{ArchetypeAccess, SystemBorrows};

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
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }

    pub fn query<'a>(&self, world: &'a World) -> QueryBorrow<'a, Q> {
        world.query::<Q>()
    }
}

impl<Q> Default for QueryEffector<Q>
where
    Q: Query + Send + Sync,
{
    fn default() -> Self {
        QueryEffector::new()
    }
}

impl<Q> Clone for QueryEffector<Q>
where
    Q: Query + Send + Sync,
{
    fn clone(&self) -> Self {
        QueryEffector::new()
    }
}

impl<Q> Copy for QueryEffector<Q> where Q: Query + Send + Sync {}

pub trait QueryUnit: Send + Sync {
    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);
}

pub trait QuerySingle: Send + Sync {
    type Effector;

    fn effector() -> Self::Effector;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess);
}

pub trait QueryBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess);
}

impl<C> QueryUnit for &'_ C
where
    C: Component,
{
    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_immutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryUnit for &'_ mut C
where
    C: Component,
{
    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_mutable.insert(TypeId::of::<C>());
    }
}

impl<Q> QueryUnit for Option<Q>
where
    Q: QueryUnit,
{
    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }
}

impl QuerySingle for () {
    type Effector = ();

    fn effector() -> Self::Effector {}

    #[cfg(feature = "parallel")]
    fn write_borrows(_: &mut SystemBorrows) {}

    #[cfg(feature = "parallel")]
    fn write_archetypes(_: &World, _: &mut ArchetypeAccess) {}
}

impl<C> QuerySingle for &'_ C
where
    C: Component,
    Self: Query + QueryUnit,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
        archetypes.extend(access_of::<Self>(world));
    }
}

impl<C> QuerySingle for &'_ mut C
where
    C: Component,
    Self: Query + QueryUnit,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
        archetypes.extend(access_of::<Self>(world));
    }
}

impl<Q> QuerySingle for Option<Q>
where
    Q: QueryUnit,
    Self: Query + QueryUnit,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
        archetypes.extend(access_of::<Self>(world));
    }
}

impl<Q> QuerySingle for (Q,)
where
    Q: QueryUnit,
    Self: Query,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
        archetypes.extend(access_of::<Self>(world));
    }
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    #[cfg(feature = "parallel")]
    fn write_borrows(_: &mut SystemBorrows) {}

    #[cfg(feature = "parallel")]
    fn write_archetypes(_: &World, _: &mut ArchetypeAccess) {}
}

impl<Q> QueryBundle for (Q,)
where
    Q: QuerySingle,
{
    type Effectors = Q::Effector;

    fn effectors() -> Self::Effectors {
        Q::effector()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
        Q::write_archetypes(world, archetypes);
    }
}

#[cfg(feature = "parallel")]
pub(crate) fn access_of<Q>(world: &World) -> impl Iterator<Item = (usize, Access)> + '_
where
    Q: Query,
{
    world
        .archetypes()
        .enumerate()
        .filter_map(|(index, archetype)| archetype.access::<Q>().map(|access| (index, access)))
}

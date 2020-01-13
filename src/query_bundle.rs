use std::{any::TypeId, marker::PhantomData};

use crate::{
    system::{ArchetypeSet, SystemBorrows},
    Component, Query, QueryBorrow, World,
};

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
    fn write_borrows(borrows: &mut SystemBorrows);
}

pub trait QuerySingle: Send + Sync {
    type Effector;

    fn effector() -> Self::Effector;

    fn write_borrows(borrows: &mut SystemBorrows);

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet);
}

pub trait QueryBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    fn write_borrows(borrows: &mut SystemBorrows);

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet);
}

impl<C> QueryUnit for &'_ C
where
    C: Component,
{
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_immutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryUnit for &'_ mut C
where
    C: Component,
{
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_mutable.insert(TypeId::of::<C>());
    }
}

impl<Q> QueryUnit for Option<Q>
where
    Q: QueryUnit,
{
    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }
}

impl QuerySingle for () {
    type Effector = ();

    fn effector() -> Self::Effector {}

    fn write_borrows(_: &mut SystemBorrows) {}

    fn write_archetypes(_: &World, _: &mut ArchetypeSet) {}
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QueryUnit>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
    }
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    fn write_borrows(_: &mut SystemBorrows) {}

    fn write_archetypes(_: &World, _: &mut ArchetypeSet) {}
}

impl<Q> QueryBundle for (Q,)
where
    Q: QuerySingle,
{
    type Effectors = Q::Effector;

    fn effectors() -> Self::Effectors {
        Q::effector()
    }

    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        Q::write_archetypes(world, archetypes);
    }
}

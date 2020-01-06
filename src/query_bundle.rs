use std::{any::TypeId, marker::PhantomData};

use crate::{borrows::ArchetypeSet, Component, Query, QueryBorrow, SystemBorrows, World};

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

    fn write_borrows(borrows: &mut SystemBorrows);

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet);
}

impl<C> QuerySingle for &'_ C
where
    C: Component,
{
    type Effector = QueryEffector<Self>;

    fn effector() -> Self::Effector {
        QueryEffector::new()
    }

    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_immutable.insert(TypeId::of::<C>());
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_mutable.insert(TypeId::of::<C>());
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
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

    fn write_borrows(borrows: &mut SystemBorrows) {
        Q::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        world.write_archetypes::<Self>(archetypes);
    }
}

pub trait QueryBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    fn write_borrows(borrows: &mut SystemBorrows);

    fn write_archetypes(world: &World, set: &mut ArchetypeSet);
}

impl<C> QueryBundle for &'_ C
where
    C: Component,
    Self: QuerySingle,
{
    type Effectors = <Self as QuerySingle>::Effector;

    fn effectors() -> Self::Effectors {
        <Self as QuerySingle>::effector()
    }

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QuerySingle>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        <Self as QuerySingle>::write_archetypes(world, archetypes);
    }
}

impl<C> QueryBundle for &'_ mut C
where
    C: Component,
    Self: QuerySingle,
{
    type Effectors = <Self as QuerySingle>::Effector;

    fn effectors() -> Self::Effectors {
        <Self as QuerySingle>::effector()
    }

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QuerySingle>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        <Self as QuerySingle>::write_archetypes(world, archetypes);
    }
}

impl<Q> QueryBundle for Option<Q>
where
    Q: QuerySingle,
    Self: QuerySingle,
{
    type Effectors = <Self as QuerySingle>::Effector;

    fn effectors() -> Self::Effectors {
        <Self as QuerySingle>::effector()
    }

    fn write_borrows(borrows: &mut SystemBorrows) {
        <Self as QuerySingle>::write_borrows(borrows);
    }

    fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
        <Self as QuerySingle>::write_archetypes(world, archetypes);
    }
}

impl QueryBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    fn write_borrows(_: &mut SystemBorrows) {}

    fn write_archetypes(_: &World, _: &mut ArchetypeSet) {}
}

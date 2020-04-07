use hecs::{Component, Query, With, Without};
use std::marker::PhantomData;

#[cfg(feature = "parallel")]
use hecs::{Access, World};
#[cfg(feature = "parallel")]
use std::any::TypeId;

use crate::fetch_components::{ComponentEffector, Immutable, Mandatory, Mutable, Optional};

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
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
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
    type ComponentEffector;

    fn component_effector() -> Self::ComponentEffector;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);
}

pub trait QuerySingle: Send + Sync {
    type QueryEffector;
    type ComponentEffectors;

    fn query_effector() -> Self::QueryEffector;

    fn component_effectors() -> Self::ComponentEffectors;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess);
}

pub trait QueryBundle: Send + Sync {
    type QueryEffectors;

    fn query_effectors() -> Self::QueryEffectors;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);

    #[cfg(feature = "parallel")]
    fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess);
}

impl<C> QueryUnit for &'_ C
where
    C: Component,
{
    type ComponentEffector = ComponentEffector<Immutable, Mandatory, C>;

    fn component_effector() -> Self::ComponentEffector {
        ComponentEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_immutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryUnit for &'_ mut C
where
    C: Component,
{
    type ComponentEffector = ComponentEffector<Mutable, Mandatory, C>;

    fn component_effector() -> Self::ComponentEffector {
        ComponentEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_mutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryUnit for Option<&'_ C>
where
    C: Component,
{
    type ComponentEffector = ComponentEffector<Immutable, Optional, C>;

    fn component_effector() -> Self::ComponentEffector {
        ComponentEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_immutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryUnit for Option<&'_ mut C>
where
    C: Component,
{
    type ComponentEffector = ComponentEffector<Mutable, Optional, C>;

    fn component_effector() -> Self::ComponentEffector {
        ComponentEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.components_mutable.insert(TypeId::of::<C>());
    }
}

impl QuerySingle for () {
    type QueryEffector = ();
    type ComponentEffectors = ();

    fn query_effector() -> Self::QueryEffector {}

    fn component_effectors() -> Self::ComponentEffectors {}

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
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = <Self as QueryUnit>::ComponentEffector;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Self::component_effector()
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
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = <Self as QueryUnit>::ComponentEffector;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Self::component_effector()
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
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = <Self as QueryUnit>::ComponentEffector;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Self::component_effector()
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

impl<C, Q> QuerySingle for With<C, Q>
where
    C: Component,
    Q: Query + QuerySingle,
{
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = Q::ComponentEffectors;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Q::component_effectors()
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

impl<C, Q> QuerySingle for Without<C, Q>
where
    C: Component,
    Q: Query + QuerySingle,
{
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = Q::ComponentEffectors;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Q::component_effectors()
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
    type QueryEffectors = ();

    fn query_effectors() -> Self::QueryEffectors {}

    #[cfg(feature = "parallel")]
    fn write_borrows(_: &mut SystemBorrows) {}

    #[cfg(feature = "parallel")]
    fn write_archetypes(_: &World, _: &mut ArchetypeAccess) {}
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

#[cfg(feature = "parallel")]
use std::any::TypeId;

use crate::Query;

#[cfg(feature = "parallel")]
use crate::{ArchetypeSet, BorrowTypeSet};

pub trait QueryExt: hecs::Query {
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet);

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet)
    where
        Self: Sized,
    {
        archetype_set.set_bits_for_query::<Self>(world);
    }
}

pub trait QueryBundle<'a> {
    fn queries(world: &'a hecs::World) -> Self;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet);

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet);
}

impl QueryExt for () {
    #[cfg(feature = "parallel")]
    fn insert_component_types(_: &mut BorrowTypeSet) {}
}

impl QueryBundle<'_> for () {
    fn queries(_: &hecs::World) -> Self {}

    #[cfg(feature = "parallel")]
    fn insert_component_types(_: &mut BorrowTypeSet) {}

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(_: &hecs::World, _: &mut ArchetypeSet) {}
}

impl<C0> QueryExt for &'_ C0
where
    C0: hecs::Component,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        component_type_set.immutable.insert(TypeId::of::<C0>());
    }
}

impl<C0> QueryExt for &'_ mut C0
where
    C0: hecs::Component,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        component_type_set.mutable.insert(TypeId::of::<C0>());
    }
}

impl<Q0> QueryExt for Option<Q0>
where
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<C0, Q0> QueryExt for hecs::With<C0, Q0>
where
    C0: hecs::Component,
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<C0, Q0> QueryExt for hecs::Without<C0, Q0>
where
    C0: hecs::Component,
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<'a, Q0> QueryBundle<'a> for Query<'a, Q0>
where
    Q0: QueryExt,
{
    fn queries(world: &'a hecs::World) -> Self {
        Query::new(world)
    }

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet) {
        Q0::set_archetype_bits(world, archetype_set);
    }
}

impl<Q0> QueryExt for (Q0,)
where
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<'a, Q0> QueryBundle<'a> for (Query<'a, Q0>,)
where
    Q0: hecs::Query + QueryExt,
{
    fn queries(world: &'a hecs::World) -> Self {
        (Query::new(world),)
    }

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q0::insert_component_types(component_type_set);
    }

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet) {
        Q0::set_archetype_bits(world, archetype_set);
    }
}

macro_rules! impl_query_ext {
    ($($letter:ident),*) => {
        impl<$($letter),*> QueryExt for ($($letter,)*)
        where
            $($letter: QueryExt,)*
        {
            #[cfg(feature = "parallel")]
            fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
                $($letter::insert_component_types(component_type_set);)*
            }
        }
    }
}

impl_for_tuples!(impl_query_ext);

macro_rules! impl_query_bundle {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> QueryBundle<'a> for ($(Query<'a, $letter>,)*)
        where
            $($letter: hecs::Query + QueryExt,)*
        {
            fn queries(world: &'a hecs::World) -> Self {
                ($(Query::<$letter>::new(world),)*)
            }

            #[cfg(feature = "parallel")]
            fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
                $($letter::insert_component_types(component_type_set);)*
            }

            #[cfg(feature = "parallel")]
            fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet) {
                $($letter::set_archetype_bits(world, archetype_set);)*
            }
        }
    }
}

impl_for_tuples!(impl_query_bundle);

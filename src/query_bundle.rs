use hecs::{Component, Query, With, Without};

#[cfg(feature = "parallel")]
use hecs::World;
#[cfg(feature = "parallel")]
use std::any::TypeId;

use crate::QueryMarker;

#[cfg(feature = "parallel")]
use crate::{ArchetypeSet, ComponentTypeSet};

pub trait QueryExt: Query {
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet);

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &World, archetype_set: &mut ArchetypeSet)
    where
        Self: Sized,
    {
        archetype_set.set_bits_for_query::<Self>(world);
    }
}

pub trait QueryBundle {
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize;

    fn markers() -> Self;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet);

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &World, archetype_set: &mut ArchetypeSet);
}

impl QueryExt for () {
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = 0;

    #[cfg(feature = "parallel")]
    fn insert_component_types(_: &mut ComponentTypeSet) {}
}

impl QueryBundle for () {
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = 0;

    fn markers() -> Self {}

    #[cfg(feature = "parallel")]
    fn insert_component_types(_: &mut ComponentTypeSet) {}

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(_: &World, _: &mut ArchetypeSet) {}
}

impl<C0> QueryExt for &'_ C0
where
    C0: Component,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = 1;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        component_type_set.immutable.insert(TypeId::of::<C0>());
    }
}

impl<C0> QueryExt for &'_ mut C0
where
    C0: Component,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = 1;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        component_type_set.mutable.insert(TypeId::of::<C0>());
    }
}

impl<Q0> QueryExt for Option<Q0>
where
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<C0, Q0> QueryExt for With<C0, Q0>
where
    C0: Component,
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<C0, Q0> QueryExt for Without<C0, Q0>
where
    C0: Component,
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<Q0> QueryBundle for QueryMarker<Q0>
where
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    fn markers() -> Self {
        QueryMarker::new()
    }

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &World, archetype_set: &mut ArchetypeSet) {
        Q0::set_archetype_bits(world, archetype_set);
    }
}

impl<Q0> QueryExt for (Q0,)
where
    Q0: QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }
}

impl<Q0> QueryBundle for (QueryMarker<Q0>,)
where
    Q0: Query + QueryExt,
{
    #[cfg(feature = "parallel")]
    const COMPONENT_TYPE_SET_LENGTH: usize = Q0::COMPONENT_TYPE_SET_LENGTH;

    fn markers() -> Self {
        (QueryMarker::new(),)
    }

    #[cfg(feature = "parallel")]
    fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
        Q0::insert_component_types(component_type_set);
    }

    #[cfg(feature = "parallel")]
    fn set_archetype_bits(world: &World, archetype_set: &mut ArchetypeSet) {
        Q0::set_archetype_bits(world, archetype_set);
    }
}

macro_rules! impl_query_single {
    ($($letter:ident),*) => {
        impl<$($letter),*> QueryExt for ($($letter,)*)
        where
            $($letter: QueryExt,)*
        {
            #[cfg(feature = "parallel")]
            const COMPONENT_TYPE_SET_LENGTH: usize = 0 $(+ $letter::COMPONENT_TYPE_SET_LENGTH)*;

            #[cfg(feature = "parallel")]
            fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
                $($letter::insert_component_types(component_type_set);)*
            }
        }
    }
}

impl_for_tuples!(impl_query_single);

macro_rules! impl_query_bundle {
    ($($letter:ident),*) => {
        impl<$($letter),*> QueryBundle for ($(QueryMarker<$letter>,)*)
        where
            $($letter: Query + QueryExt,)*
        {
            #[cfg(feature = "parallel")]
            const COMPONENT_TYPE_SET_LENGTH: usize = 0 $(+ $letter::COMPONENT_TYPE_SET_LENGTH)*;

            fn markers() -> Self {
                ($(QueryMarker::<$letter>::new(),)*)
            }

            #[cfg(feature = "parallel")]
            fn insert_component_types(component_type_set: &mut ComponentTypeSet) {
                $($letter::insert_component_types(component_type_set);)*
            }

            #[cfg(feature = "parallel")]
            fn set_archetype_bits(world: &World, archetype_set: &mut ArchetypeSet) {
                $($letter::set_archetype_bits(world, archetype_set);)*
            }
        }
    }
}

impl_for_tuples!(impl_query_bundle);

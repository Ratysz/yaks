#[cfg(feature = "parallel")]
use std::any::TypeId;

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

impl QueryExt for () {
    #[cfg(feature = "parallel")]
    fn insert_component_types(_: &mut BorrowTypeSet) {}
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

impl_for_tuples!(impl_query_ext, all);

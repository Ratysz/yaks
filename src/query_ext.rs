use std::any::TypeId;

use crate::{ArchetypeSet, BorrowTypeSet};

pub trait QueryExt: hecs::Query {
    fn insert_component_types(component_type_set: &mut BorrowTypeSet);

    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet)
    where
        Self: Sized,
    {
        archetype_set.set_bits_for_query::<Self>(world);
    }
}

impl QueryExt for () {
    fn insert_component_types(_: &mut BorrowTypeSet) {}
}

impl<C> QueryExt for &'_ C
where
    C: hecs::Component,
{
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        component_type_set.immutable.insert(TypeId::of::<C>());
    }
}

impl<C> QueryExt for &'_ mut C
where
    C: hecs::Component,
{
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        component_type_set.mutable.insert(TypeId::of::<C>());
    }
}

impl<Q> QueryExt for Option<Q>
where
    Q: QueryExt,
{
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q::insert_component_types(component_type_set);
    }
}

impl<C, Q> QueryExt for hecs::With<C, Q>
where
    C: hecs::Component,
    Q: QueryExt,
{
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q::insert_component_types(component_type_set);
    }
}

impl<C, Q> QueryExt for hecs::Without<C, Q>
where
    C: hecs::Component,
    Q: QueryExt,
{
    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        Q::insert_component_types(component_type_set);
    }
}

macro_rules! impl_query_ext {
    ($($letter:ident),*) => {
        impl<$($letter),*> QueryExt for ($($letter,)*)
        where
            $($letter: QueryExt,)*
        {
            fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
                $($letter::insert_component_types(component_type_set);)*
            }
        }
    }
}

impl_for_tuples!(impl_query_ext, all);

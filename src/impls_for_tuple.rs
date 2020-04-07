use hecs::{Component, EntityRef, Query, World};
use resources::{Resource, Resources};

use crate::{
    fetch_components::{ComponentEffector, Fetch as ComponentFetch, Mutability, Optionality},
    query_bundle::{QueryBundle, QueryEffector, QuerySingle, QueryUnit},
    resource_bundle::{Fetch as ResourceFetch, ResourceBundle, ResourceEffector, ResourceSingle},
    system::TupleAppend,
};

#[cfg(feature = "parallel")]
use crate::{query_bundle::access_of, ArchetypeAccess, SystemBorrows};

impl<T0, T1> TupleAppend<T1> for (T0,) {
    type Output = (T0, T1);
}

impl<R> ResourceBundle for (R,)
where
    R: ResourceSingle,
{
    type Effectors = R::Effector;

    fn effectors() -> Self::Effectors {
        R::effector()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        R::write_borrows(borrows)
    }
}

impl<Q> QuerySingle for (Q,)
where
    Q: QueryUnit,
    Self: Query,
{
    type QueryEffector = QueryEffector<Self>;
    type ComponentEffectors = Q::ComponentEffector;

    fn query_effector() -> Self::QueryEffector {
        QueryEffector::new()
    }

    fn component_effectors() -> Self::ComponentEffectors {
        Q::component_effector()
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

impl<Q> QueryBundle for (Q,)
where
    Q: QuerySingle,
{
    type QueryEffectors = Q::QueryEffector;

    fn query_effectors() -> Self::QueryEffectors {
        Q::query_effector()
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

macro_rules! impls_for_tuple {
    ($($letter:ident),*) => {
        impl<$($letter),* , Input> TupleAppend<Input> for ($($letter,)*)
        {
            type Output = ($($letter,)* Input);
        }

        impl<$($letter),*> ResourceBundle for ($($letter,)*)
        where
            $($letter: ResourceSingle,)*
        {
            type Effectors = ($($letter::Effector,)*);

            fn effectors() -> Self::Effectors {
                ($($letter::effector(),)*)
            }

            #[cfg(feature = "parallel")]
            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }
        }

        paste::item! {
            impl<'a, $([<M $letter>]),*, $([<R $letter>]),*> ResourceFetch<'a>
                for ($(ResourceEffector<[<M $letter>], [<R $letter>]>,)*)
            where
                $([<M $letter>]: Mutability,)*
                $([<R $letter>]: Resource,)*
                $(ResourceEffector<[<M $letter>], [<R $letter>]>: ResourceFetch<'a>,)*
            {
                type Refs = (
                    $(<ResourceEffector<[<M $letter>], [<R $letter>]> as ResourceFetch<'a>>::Refs,)*
                );

                fn fetch(&self, resources: &'a Resources) -> Self::Refs {
                    ($(ResourceEffector::<[<M $letter>], [<R $letter>]>::new().fetch(resources),)*)
                }
            }
        }

        paste::item! {
            impl<'a, $([<M $letter>]),*, $([<O $letter>]),*, $([<C $letter>]),*> ComponentFetch<'a>
                for ($(ComponentEffector<[<M $letter>], [<O $letter>], [<C $letter>]>,)*)
            where
                $([<M $letter>]: Mutability,)*
                $([<O $letter>]: Optionality,)*
                $([<C $letter>]: Component,)*
                $(ComponentEffector<[<M $letter>], [<O $letter>], [<C $letter>]>:
                    ComponentFetch<'a>,)*
            {
                type Refs = (
                    $(<ComponentEffector<[<M $letter>], [<O $letter>], [<C $letter>]>
                        as ComponentFetch<'a>>::Refs,)*
                );

                fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs {
                    ($(ComponentEffector::<[<M $letter>], [<O $letter>], [<C $letter>]>::new()
                        .fetch(entity_ref),)*)
                }
            }
        }

        impl<$($letter),*> QuerySingle for ($($letter,)*)
        where
            $($letter: QueryUnit,)*
            Self: Query,
        {
            type QueryEffector = QueryEffector<Self>;
            type ComponentEffectors = ($($letter::ComponentEffector,)*);

            fn query_effector() -> Self::QueryEffector {
                QueryEffector::new()
            }

            fn component_effectors() -> Self::ComponentEffectors {
                ($($letter::component_effector(),)*)
            }

            #[cfg(feature = "parallel")]
            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            #[cfg(feature = "parallel")]
            fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
                archetypes.extend(access_of::<Self>(world));
            }
        }

        impl<$($letter),*> QueryBundle for ($($letter,)*)
        where
            $($letter: Query + QuerySingle + Send + Sync,)*
        {
            type QueryEffectors = ($($letter::QueryEffector,)*);

            fn query_effectors() -> Self::QueryEffectors {
                ($($letter::query_effector(),)*)
            }

            #[cfg(feature = "parallel")]
            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            #[cfg(feature = "parallel")]
            fn write_archetypes(world: &World, archetypes: &mut ArchetypeAccess) {
                archetypes.extend(world
                    .archetypes()
                    .enumerate()
                    .filter_map(|(index, archetype)|
                        None
                            $( .or_else(|| archetype.access::<$letter>()) )*
                            .map(|access| (index, access))
                    )
                );
            }
        }
    };
}

macro_rules! expand {
    ($macro:ident, $letter:ident) => {
        //$macro!($letter);
    };
    ($macro:ident, $letter:ident, $($tail:ident),*) => {
        $macro!($letter, $($tail),*);
        expand!($macro, $($tail),*);
    };
}

#[rustfmt::skip]
expand!(impls_for_tuple, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);

use crate::{
    borrows::{ArchetypeSet, SystemBorrows},
    query_bundle::{QueryBundle, QueryEffector, QuerySingle, QueryUnit},
    resource_bundle::{Fetch, Mutability, ResourceBundle, ResourceEffector, ResourceSingle},
    Query, Resource, World,
};

pub trait TupleAppend<T> {
    type Output;
}

impl<T> TupleAppend<T> for () {
    type Output = (T,);
}

impl<T0, T1> TupleAppend<T1> for (T0,) {
    type Output = (T0, T1);
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

            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }
        }

        paste::item! {
            impl<'a, $([<M $letter>]),*, $([<R $letter>]),*> Fetch<'a>
                for ($(ResourceEffector<[<M $letter>], [<R $letter>]>,)*)
            where
                $([<M $letter>]: Mutability,)*
                $([<R $letter>]: Resource,)*
                $(ResourceEffector<[<M $letter>], [<R $letter>]>: Fetch<'a>,)*
            {
                type Refs = (
                    $(<ResourceEffector<[<M $letter>], [<R $letter>]> as Fetch<'a>>::Refs,)*
                );

                fn fetch(&self, world: &'a World) -> Self::Refs {
                    ($(ResourceEffector::<[<M $letter>], [<R $letter>]>::new().fetch(world),)*)
                }
            }
        }

        impl<$($letter),*> QuerySingle for ($($letter,)*)
        where
            $($letter: QueryUnit,)*
            Self: Query,
        {
            type Effector = QueryEffector<Self>;

            fn effector() -> Self::Effector {
                QueryEffector::new()
            }

            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
                world.write_archetypes::<Self>(archetypes);
            }
        }

        impl<$($letter),*> QueryBundle for ($($letter,)*)
        where
            $($letter: QuerySingle + Send + Sync,)*
        {
            type Effectors = ($($letter::Effector,)*);

            fn effectors() -> Self::Effectors {
                ($($letter::effector(),)*)
            }

            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            fn write_archetypes(world: &World, archetypes: &mut ArchetypeSet) {
                $($letter::write_archetypes(world, archetypes);)*
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

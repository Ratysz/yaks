use hecs::Query;
use resources::{Resource, Resources};

#[cfg(feature = "parallel")]
use hecs::World;

use crate::{
    query_bundle::{QueryBundle, QueryEffector, QuerySingle, QueryUnit},
    resource_bundle::{Fetch, Mutability, ResourceBundle, ResourceEffector, ResourceSingle},
    system::TupleAppend,
};

#[cfg(feature = "parallel")]
use crate::borrows::{ArchetypeSet, SystemBorrows};

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

                fn fetch(&self, resources: &'a Resources) -> Self::Refs {
                    ($(ResourceEffector::<[<M $letter>], [<R $letter>]>::new().fetch(resources),)*)
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

            #[cfg(feature = "parallel")]
            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            #[cfg(feature = "parallel")]
            fn write_archetypes(_world: &World, _archetypes: &mut ArchetypeSet) {
                // TODO world.write_archetypes::<Self>(archetypes);
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

            #[cfg(feature = "parallel")]
            fn write_borrows(borrows: &mut SystemBorrows) {
                $($letter::write_borrows(borrows);)*
            }

            #[cfg(feature = "parallel")]
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

use crate::{
    query_bundle::{QueryBundle, QueryEffector, QuerySingle},
    resource_bundle::{Fetch, Mutability, ResourceBundle, ResourceEffector, ResourceSingle},
    system::ArchetypeSet,
    Query, Resource, SystemMetadata, World,
};

macro_rules! impls_for_tuple {
    ($($letter:ident),*) => {
        impl<$($letter),*> ResourceBundle for ($($letter,)*)
        where
            $($letter: ResourceSingle,)*
        {
            type Effectors = ($($letter::Effector,)*);

            fn effectors() -> Self::Effectors {
                ($($letter::effector(),)*)
            }

            fn write_metadata(metadata: &mut SystemMetadata) {
                $($letter::write_metadata(metadata);)*
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
            $($letter: QuerySingle,)*
            Self: Query,
        {
            type Effector = QueryEffector<Self>;

            fn effector() -> Self::Effector {
                QueryEffector::new()
            }

            fn write_metadata(metadata: &mut SystemMetadata) {
                $($letter::write_metadata(metadata);)*
            }

            fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
                world.write_touched_archetypes::<Self>(set);
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

            fn write_metadata(metadata: &mut SystemMetadata) {
                $($letter::write_metadata(metadata);)*
            }

            fn write_touched_archetypes(world: &World, set: &mut ArchetypeSet) {
                $($letter::write_touched_archetypes(world, set);)*
            }
        }
    };
}

macro_rules! expand {
    ($macro:ident, $letter:ident) => {
        $macro!($letter);
    };
    ($macro:ident, $letter:ident, $($tail:ident),*) => {
        $macro!($letter, $($tail),*);
        expand!($macro, $($tail),*);
    };
}

#[rustfmt::skip]
expand!(impls_for_tuple, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);

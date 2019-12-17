use crate::{
    query_bundle::{QueryBundle, QueryEffector},
    resource_bundle::{FetchEffector, Mutability, ResourceBundle},
    system::TypeSet,
    world::ArchetypeSet,
    Fetch, Query, Resource, World,
};

macro_rules! impls_for_tuple {
    ($($letter:ident),*) => {
        impl<$($letter),*> ResourceBundle for ($($letter,)*)
        where
            $($letter: Resource + ResourceBundle,)*
        {
            type Effectors = ($($letter::Effectors,)*);

            fn effectors() -> Self::Effectors {
                ($($letter::effectors(),)*)
            }
        }

        paste::item! {
            impl<'a, $([<M $letter>]),*, $([<R $letter>]),*> Fetch<'a>
                for ($(FetchEffector<[<M $letter>], [<R $letter>]>,)*)
            where
                $([<M $letter>]: Mutability,)*
                $([<R $letter>]: Resource,)*
                $(FetchEffector<[<M $letter>], [<R $letter>]>: Fetch<'a>,)*
            {
                type Refs = ($(<FetchEffector<[<M $letter>], [<R $letter>]> as Fetch<'a>>::Refs,)*);

                fn fetch(&self, world: &'a World) -> Self::Refs {
                    ($(FetchEffector::<[<M $letter>], [<R $letter>]>::new().fetch(world),)*)
                }
            }
        }

        // FIXME this should be used instead of the above after
        //  https://github.com/rust-lang/rust/issues/62529 is fixed
        /*impl<'a, $($letter: ResourceBundle),*> ResourceBundle for ($($letter,)*)
        {
            type Refs = ($($letter::Refs,)*);
        }

        impl<'a, $($letter: Fetch<'a>),*> Fetch<'a> for ($($letter,)*)
        {
            type Item = ($($letter::Item,)*);

            fn fetch(world: &'a World) -> Self::Item {
                ($($letter::fetch(world),)*)
            }
        }*/

        impl<$($letter),*> QueryBundle for ($($letter,)*)
        where
            $($letter: Query + QueryBundle,)*
        {
            type Effectors = ($(QueryEffector<$letter>,)*);

            fn effectors() -> Self::Effectors {
                ($(QueryEffector::<$letter>::new(),)*)
            }

            fn borrowed_components() -> TypeSet {
                let mut set = TypeSet::default();
                $(set.extend($letter::borrowed_components().drain());)*
                set
            }

            fn borrowed_mut_components() -> TypeSet {
                let mut set = TypeSet::default();
                $(set.extend($letter::borrowed_mut_components().drain());)*
                set
            }

            fn touched_archetypes(world: &World) -> ArchetypeSet {
                let mut set = ArchetypeSet::default();
                $(set.extend($letter::touched_archetypes(world).drain());)*
                set
            }
        }
    };
}

macro_rules! expand {
    ($m: ident, $ty: ident) => {
        $m!{$ty}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        $m!{$ty, $($tt),*}
        expand!{$m, $($tt),*}
    };
}

#[rustfmt::skip]
expand!(impls_for_tuple, P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);

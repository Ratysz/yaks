use crate::{
    query_bundle::{QueryBundle, QueryEffector},
    resource_bundle::{Fetch, ResourceBundle},
    system::TypeSet,
    world::ArchetypeSet,
    Query, World,
};

macro_rules! impls_for_tuple {
    ($($letter:ident),*) => {
        impl<'a, $($letter: ResourceBundle),*> ResourceBundle for ($($letter,)*)
        {
            type Refs = ($($letter::Refs,)*);
        }

        impl<'a, $($letter: Fetch<'a>),*> Fetch<'a> for ($($letter,)*)
        {
            type Item = ($($letter::Item,)*);

            fn fetch(world: &'a World) -> Self::Item {
                ($($letter::fetch(world),)*)
            }
        }

        impl<$($letter: Query + QueryBundle),*> QueryBundle for ($($letter,)*)
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

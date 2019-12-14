use crate::{
    query_bundle::{QueryBundle, QueryEffector},
    resource_bundle::ResourceBundle,
    Query, World,
};

macro_rules! impls_for_tuple {
    ($($letter:ident),*) => {
        impl<'a, $($letter: ResourceBundle<'a>),*> ResourceBundle<'a> for ($($letter,)*)
        {
            type Refs = ($($letter::Refs,)*);

            fn fetch(world: &'a World) -> Self::Refs {
                ($($letter::fetch(world),)*)
             }
        }

        impl<'a, $($letter: Query<'a> + Send + Sync),*> QueryBundle<'a> for ($($letter,)*)
        {
            type QueryEffectors = ($(QueryEffector<'a, $letter>,)*);

            fn query_effectors() -> Self::QueryEffectors {
                ($(QueryEffector::<'a, $letter>::new(),)*)
             }
        }
    };
}

impls_for_tuple!(A);
impls_for_tuple!(A, B);
impls_for_tuple!(A, B, C);
impls_for_tuple!(A, B, C, D);
impls_for_tuple!(A, B, C, D, E);
impls_for_tuple!(A, B, C, D, E, F);
impls_for_tuple!(A, B, C, D, E, F, G);
impls_for_tuple!(A, B, C, D, E, F, G, H);
impls_for_tuple!(A, B, C, D, E, F, G, H, I);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impls_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

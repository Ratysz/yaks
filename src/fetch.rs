use std::any::type_name;

use crate::{Resource, ResourceRef, ResourceRefMut, World};

pub trait Fetch<'a>: Send + Sync {
    type Refs;

    fn fetch(world: &'a World) -> Self::Refs;
}

impl<'a, R: Resource> Fetch<'a> for &'a R {
    type Refs = ResourceRef<'a, R>;

    fn fetch(world: &'a World) -> Self::Refs {
        world
            .resource()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl<'a, R: Resource> Fetch<'a> for &'a mut R {
    type Refs = ResourceRefMut<'a, R>;

    fn fetch(world: &'a World) -> Self::Refs {
        world
            .resource_mut()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

macro_rules! impl_resource_bundle_for_tuple {
    ($($bundle:ident),*) => {
        impl<'a, $($bundle: Fetch<'a>),*> Fetch<'a> for ($($bundle,)*)
        {
            type Refs = ($($bundle::Refs,)*);

            fn fetch(world: &'a World) -> Self::Refs {
                ($( $bundle::fetch(world),)*)
             }
        }
    };
}

impl_resource_bundle_for_tuple!(A);
impl_resource_bundle_for_tuple!(A, B);
impl_resource_bundle_for_tuple!(A, B, C);
impl_resource_bundle_for_tuple!(A, B, C, D);
impl_resource_bundle_for_tuple!(A, B, C, D, E);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_resource_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

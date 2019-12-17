use std::{any::type_name, marker::PhantomData};

use crate::{Resource, ResourceRef, ResourceRefMut, World};

pub trait ResourceBundle: Send + Sync {
    type Refs: for<'a> Fetch<'a>;

    fn fetch(world: &World) -> <Self::Refs as Fetch>::Item {
        Self::Refs::fetch(world)
    }
}

pub trait Fetch<'a> {
    type Item;

    fn fetch(world: &'a World) -> Self::Item;
}

pub struct ElidedRef<R: Resource> {
    phantom_data: PhantomData<R>,
}

pub struct ElidedRefMut<R: Resource> {
    phantom_data: PhantomData<R>,
}

impl<'a, R> ResourceBundle for &'a R
where
    R: Resource,
{
    type Refs = ElidedRef<R>;
}

impl<'a, R> Fetch<'a> for ElidedRef<R>
where
    R: Resource,
{
    type Item = ResourceRef<'a, R>;

    fn fetch(world: &'a World) -> Self::Item {
        world
            .resource()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl<'a, R: Resource> ResourceBundle for &'a mut R {
    type Refs = ElidedRefMut<R>;
}

impl<'a, R> Fetch<'a> for ElidedRefMut<R>
where
    R: Resource,
{
    type Item = ResourceRefMut<'a, R>;

    fn fetch(world: &'a World) -> Self::Item {
        world
            .resource_mut()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl ResourceBundle for () {
    type Refs = ();
}

impl<'a> Fetch<'a> for () {
    type Item = ();

    fn fetch(_: &'a World) -> Self::Item {}
}

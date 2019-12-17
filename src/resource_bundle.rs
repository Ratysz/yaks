use std::{any::type_name, marker::PhantomData};

use crate::{Resource, ResourceRef, ResourceRefMut, World};

pub struct Immutable;

pub struct Mutable;

pub trait Mutability: Send + Sync {}

impl Mutability for Immutable {}

impl Mutability for Mutable {}

pub struct FetchEffector<M, R>
where
    M: Mutability,
    R: Resource + Send + Sync,
{
    phantom_data: PhantomData<(M, R)>,
}

impl<M, R> FetchEffector<M, R>
where
    M: Mutability,
    R: Resource + Send + Sync,
{
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

pub trait ResourceBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;
}

pub trait Fetch<'a> {
    type Refs;

    fn fetch(&self, world: &'a World) -> Self::Refs;
}

impl ResourceBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}
}

impl<'a> Fetch<'a> for () {
    type Refs = ();

    fn fetch(&self, world: &'a World) -> Self::Refs {}
}

impl<R: Resource> ResourceBundle for &'_ R {
    type Effectors = FetchEffector<Immutable, R>;

    fn effectors() -> Self::Effectors {
        FetchEffector::new()
    }
}

impl<'a, R: Resource> Fetch<'a> for FetchEffector<Immutable, R> {
    type Refs = ResourceRef<'a, R>;

    fn fetch(&self, world: &'a World) -> Self::Refs {
        world
            .resource()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl<R: Resource> ResourceBundle for &'_ mut R {
    type Effectors = FetchEffector<Mutable, R>;

    fn effectors() -> Self::Effectors {
        FetchEffector::new()
    }
}

impl<'a, R: Resource> Fetch<'a> for FetchEffector<Mutable, R> {
    type Refs = ResourceRefMut<'a, R>;

    fn fetch(&self, world: &'a World) -> Self::Refs {
        world
            .resource_mut()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

// FIXME this should be used instead of the above after
//  https://github.com/rust-lang/rust/issues/62529 is fixed
/*
pub struct ElidedRef<R: Resource> {
    phantom_data: PhantomData<R>,
}

pub struct ElidedRefMut<R: Resource> {
    phantom_data: PhantomData<R>,
}

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

impl ResourceBundle for () {
    type Refs = ();
}

impl<'a> Fetch<'a> for () {
    type Item = ();

    fn fetch(_: &'a World) -> Self::Item {}
}

impl<R> ResourceBundle for &'_ R
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

impl<R: Resource> ResourceBundle for &'_ mut R {
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
*/

use resources::{Ref, RefMut, Resource, Resources};
use std::{any::type_name, marker::PhantomData};

#[cfg(feature = "parallel")]
use std::any::TypeId;

#[cfg(feature = "parallel")]
use crate::borrows::SystemBorrows;

pub struct Immutable;

pub struct Mutable;

pub trait Mutability: Send + Sync {}

impl Mutability for Immutable {}

impl Mutability for Mutable {}

pub struct ResourceEffector<M, R>
where
    M: Mutability,
    R: Resource + Send + Sync,
{
    phantom_data: PhantomData<(M, R)>,
}

impl<M, R> ResourceEffector<M, R>
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

pub trait ResourceSingle: Send + Sync {
    type Effector;

    fn effector() -> Self::Effector;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);
}

pub trait ResourceBundle: Send + Sync {
    type Effectors;

    fn effectors() -> Self::Effectors;

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows);
}

pub trait Fetch<'a> {
    type Refs;

    fn fetch(&self, resources: &'a Resources) -> Self::Refs;
}

impl<R> ResourceSingle for &'_ R
where
    R: Resource,
{
    type Effector = ResourceEffector<Immutable, R>;

    fn effector() -> Self::Effector {
        ResourceEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.resources_immutable.insert(TypeId::of::<R>());
    }
}

impl<R> ResourceSingle for &'_ mut R
where
    R: Resource,
{
    type Effector = ResourceEffector<Mutable, R>;

    fn effector() -> Self::Effector {
        ResourceEffector::new()
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(borrows: &mut SystemBorrows) {
        borrows.resources_mutable.insert(TypeId::of::<R>());
    }
}

impl ResourceBundle for () {
    type Effectors = ();

    fn effectors() -> Self::Effectors {}

    #[cfg(feature = "parallel")]
    fn write_borrows(_: &mut SystemBorrows) {}
}

impl<R> ResourceBundle for R
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

impl<'a> Fetch<'a> for () {
    type Refs = ();

    fn fetch(&self, _: &'a Resources) -> Self::Refs {}
}

impl<'a, R> Fetch<'a> for ResourceEffector<Immutable, R>
where
    R: Resource,
{
    type Refs = Ref<'a, R>;

    fn fetch(&self, resources: &'a Resources) -> Self::Refs {
        resources
            .get()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl<'a, R> Fetch<'a> for ResourceEffector<Mutable, R>
where
    R: Resource,
{
    type Refs = RefMut<'a, R>;

    fn fetch(&self, resources: &'a Resources) -> Self::Refs {
        resources
            .get_mut()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

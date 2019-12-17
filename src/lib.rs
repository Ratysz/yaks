// TODO uncomment #![warn(missing_docs)]

use fxhash::FxHasher64;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault};

pub use hecs::{
    Bundle as ComponentBundle, Component, ComponentError, DynamicBundle as DynamicComponentBundle,
    Entity, EntityRef as Components, MissingComponent, NoSuchEntity, Query, QueryBorrow,
    Ref as ComponentRef, RefMut as ComponentRefMut,
};
pub use resources::{
    CantGetResource as ResourceError, Entry as ResourceEntry, NoSuchResource, Ref as ResourceRef,
    RefMut as ResourceRefMut, Resource,
};

mod impls_for_tuple;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;

pub use system::System;
pub use world::World;

pub(crate) type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;
pub(crate) type ArchetypeSet = HashSet<u32, BuildHasherDefault<FxHasher64>>;

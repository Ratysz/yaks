// TODO uncomment #![warn(missing_docs)]

#[doc(hidden)]
pub use hecs::{
    Bundle as ComponentBundle, DynamicBundle as DynamicComponentBundle, EntityRef as Components,
    Query, Ref as ComponentRef, RefMut as ComponentRefMut,
};
#[doc(hidden)]
pub use resources::{Entry as ResourceEntry, Ref as ResourceRef, RefMut as ResourceRefMut};

pub use hecs::{Component, Entity, QueryBorrow};
pub use resources::Resource;

pub mod error;
mod executor;
mod impls_for_tuple;
mod mod_queue;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;

pub use executor::Executor;
pub use mod_queue::ModQueue;
pub use system::{System, SystemBuilder};
pub use world::World;

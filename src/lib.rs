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

mod executor;
//mod executor_arch_disjoint;
pub mod error;
mod impls_for_tuple;
mod modification_queue;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;
mod world_proxy;

pub use executor::{Executor, SystemHandle};
pub use modification_queue::ModificationQueue;
pub use query_bundle::QueryEffector;
pub use system::{System, SystemBuilder};
pub use world::World;
pub use world_proxy::WorldProxy;

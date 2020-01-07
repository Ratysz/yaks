// TODO uncomment #![warn(missing_docs)]

pub use hecs::{
    Bundle as ComponentBundle, Component, ComponentError, DynamicBundle as DynamicComponentBundle,
    Entity, EntityRef as Components, MissingComponent, NoSuchEntity, Query, QueryBorrow,
    Ref as ComponentRef, RefMut as ComponentRefMut,
};
pub use resources::{
    CantGetResource as ResourceError, Entry as ResourceEntry, NoSuchResource, Ref as ResourceRef,
    RefMut as ResourceRefMut, Resource,
};

mod borrows;
mod executor;
//mod executor_arch_disjoint;
mod error;
mod impls_for_tuple;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;
mod world_proxy;

pub use error::{NoSuchSystem, NonUniqueSystemHandle};
pub use executor::{Executor, SystemHandle};
pub use system::System;
pub use world::World;
pub use world_proxy::WorldProxy;

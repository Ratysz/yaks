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

mod impls_for_tuple;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;

pub use resource_bundle::Fetch;
pub use system::{DynamicSystemBuilder, StaticSystem, StaticSystemBuilder};
pub use world::World;

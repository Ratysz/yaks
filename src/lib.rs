// TODO uncomment #![warn(missing_docs)]

pub use hecs::{
    Bundle as ComponentBundle, Component, ComponentError, DynamicBundle as DynamicComponentBundle,
    Entity, EntityRef as Components, MissingComponent, NoSuchEntity, Query, QueryIter,
    Ref as ComponentRef, RefMut as ComponentRefMut,
};
pub use resources::{
    CantGetResource as ResourceError, Entry as ResourceEntry, NoSuchResource, Ref as ResourceRef,
    RefMut as ResourceRefMut, Resource,
};

mod fetch;
mod system;
mod world;

pub use world::World;

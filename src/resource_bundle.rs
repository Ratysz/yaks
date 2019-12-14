use std::any::type_name;

use crate::{Resource, ResourceRef, ResourceRefMut, World};

pub trait ResourceBundle<'a>: Send + Sync {
    type Refs;

    fn fetch(world: &'a World) -> Self::Refs;
}

impl<'a, R: Resource> ResourceBundle<'a> for &'a R {
    type Refs = ResourceRef<'a, R>;

    fn fetch(world: &'a World) -> Self::Refs {
        world
            .resource()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

impl<'a, R: Resource> ResourceBundle<'a> for &'a mut R {
    type Refs = ResourceRefMut<'a, R>;

    fn fetch(world: &'a World) -> Self::Refs {
        world
            .resource_mut()
            .unwrap_or_else(|error| panic!("cannot fetch {}: {}", type_name::<R>(), error))
    }
}

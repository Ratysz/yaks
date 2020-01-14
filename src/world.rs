use hecs::World as Entities;
use resources::Resources;

use crate::{
    error::{ComponentError, NoSuchEntity, NoSuchResource, ResourceError},
    mod_queue::{ModQueue, ModQueuePool},
    resource_bundle::{Fetch, ResourceBundle},
    system::ArchetypeSet,
    Component, ComponentBundle, ComponentRef, ComponentRefMut, Components, DynamicComponentBundle,
    Entity, Query, QueryBorrow, Resource, ResourceEntry, ResourceRef, ResourceRefMut,
};

/// A collection of [`Entities`], each with any number of associated [`Components`],
/// and [`Resources`], which are components that are not associated with any particular entity.
///
/// More information: [`hecs::World`], [`resources::Resources`].
///
/// [`Entities`]: struct.Entity.html
/// [`Components`]: trait.Component.html
/// [`Resources`]: trait.Resource.html
/// [`hecs::World`]: ../hecs/struct.World.html
/// [`resources::Resources`]: ../resources/struct.Resources.html
#[derive(Default)]
pub struct World {
    entities: Entities,
    resources: Resources,
    mod_queues: ModQueuePool,
}

impl World {
    /// Creates an empty world. Functionally identical to [`::default()`](#method.default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an entity with `components`, returning it's ID. Components can be given as
    /// any tuple, structs that derive [`ComponentBundle`](trait.ComponentBundle.html),
    /// or an [`EntityBuilder`](struct.EntityBuilder.html).
    pub fn spawn<C>(&mut self, components: C) -> Entity
    where
        C: DynamicComponentBundle,
    {
        self.entities.spawn(components)
    }

    /*pub fn reserve(&self) -> Entity {
        self.entities.reserve()
    }*/

    /// Destroy the `entity` and all of it's components.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.entities.despawn(entity)
    }

    /// Destroy all entities and all of their components.
    pub fn despawn_all(&mut self) {
        self.entities.clear();
    }

    /// Returns `true` if the given entity exists in the `World`.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    /// Adds the `components` to the `entity`. Components can be given as any tuple,
    /// structs that derive [`ComponentBundle`](trait.ComponentBundle.html),
    /// or an [`EntityBuilder`](struct.EntityBuilder.html).
    pub fn add_components<C>(&mut self, entity: Entity, components: C) -> Result<(), NoSuchEntity>
    where
        C: DynamicComponentBundle,
    {
        self.entities.insert(entity, components)
    }

    /// Removes components from the `entity`. Components can be specified by any tuple,
    /// or structs that derive [`ComponentBundle`](trait.ComponentBundle.html).
    pub fn remove_components<C>(&mut self, entity: Entity) -> Result<C, ComponentError>
    where
        C: ComponentBundle,
    {
        self.entities.remove(entity)
    }

    /// Returns a borrow of the `C` component of `entity`.
    /// Panics if it is already borrowed in an incompatible way.
    pub fn component<C>(&self, entity: Entity) -> Result<ComponentRef<C>, ComponentError>
    where
        C: Component,
    {
        self.entities.get::<C>(entity)
    }

    /// Returns a mutable borrow of the `C` component of `entity`.
    /// Panics if it is already borrowed in an incompatible way.
    pub fn component_mut<C>(&self, entity: Entity) -> Result<ComponentRefMut<C>, ComponentError>
    where
        C: Component,
    {
        self.entities.get_mut::<C>(entity)
    }

    /// Returns a [`Components`](struct.Components.html) struct that can be used to access
    /// components of the `entity`. Does not immediately borrow any components.
    pub fn components(&self, entity: Entity) -> Result<Components, NoSuchEntity> {
        self.entities.entity(entity)
    }

    pub fn query<Q>(&self) -> QueryBorrow<Q>
    where
        Q: Query,
    {
        self.entities.query()
    }

    pub fn add_resource<R>(&mut self, resource: R) -> Option<R>
    where
        R: Resource,
    {
        self.resources.insert(resource)
    }

    pub fn remove_resource<R>(&mut self) -> Result<R, NoSuchResource>
    where
        R: Resource,
    {
        self.resources.remove().ok_or_else(|| NoSuchResource)
    }

    pub fn resource_entry<R>(&mut self) -> ResourceEntry<R>
    where
        R: Resource,
    {
        self.resources.entry()
    }

    pub fn contains_resource<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains::<R>()
    }

    pub fn resource<R>(&self) -> Result<ResourceRef<R>, ResourceError>
    where
        R: Resource,
    {
        self.resources.get()
    }

    pub fn resource_mut<R>(&self) -> Result<ResourceRefMut<R>, ResourceError>
    where
        R: Resource,
    {
        self.resources.get_mut()
    }

    pub fn fetch<'a, RB>(&'a self) -> <RB::Effectors as Fetch<'a>>::Refs
    where
        RB: ResourceBundle,
        RB::Effectors: Fetch<'a>,
    {
        RB::effectors().fetch(self)
    }

    pub fn new_mod_queue(&self) -> ModQueue {
        self.mod_queues.get()
    }

    pub fn flush_mod_queues(&mut self) {
        //self.entities.flush();
        if let Some(mut queue) = self.mod_queues.flatten() {
            queue.apply_all(self);
        }
    }

    pub(crate) fn write_archetypes<Q: Query>(&self, _archetypes: &mut ArchetypeSet) {
        //println!("archetype handling is not implemented yet");
        //archetypes.extend(self.entities.query_scope::<Q>());
    }
}

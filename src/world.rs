use hecs::World as Entities;
use resources::Resources;
use std::ops::RangeBounds;

use crate::{
    modification_queue::{ModificationQueue, ModificationQueuePool},
    resource_bundle::{Fetch, ResourceBundle},
    system::ArchetypeSet,
    Component, ComponentBundle, ComponentError, ComponentRef, ComponentRefMut, Components,
    DynamicComponentBundle, Entity, NoSuchEntity, NoSuchResource, Query, QueryBorrow, Resource,
    ResourceEntry, ResourceError, ResourceRef, ResourceRefMut,
};

#[derive(Default)]
pub struct World {
    entities: Entities,
    resources: Resources,
    modification_queues: ModificationQueuePool,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, components: impl DynamicComponentBundle) -> Entity {
        self.entities.spawn(components)
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.entities.despawn(entity)
    }

    pub fn despawn_all(&mut self) {
        self.entities.clear();
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn add_components(
        &mut self,
        entity: Entity,
        components: impl DynamicComponentBundle,
    ) -> Result<(), NoSuchEntity> {
        self.entities.insert(entity, components)
    }

    pub fn remove_components<T: ComponentBundle>(
        &mut self,
        entity: Entity,
    ) -> Result<T, ComponentError> {
        self.entities.remove(entity)
    }

    pub fn component<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<ComponentRef<C>, ComponentError> {
        self.entities.get::<C>(entity)
    }

    pub fn component_mut<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<ComponentRefMut<C>, ComponentError> {
        self.entities.get_mut::<C>(entity)
    }

    pub fn components(&self, entity: Entity) -> Result<Components, NoSuchEntity> {
        self.entities.entity(entity)
    }

    pub fn query<Q: Query>(&self) -> QueryBorrow<Q> {
        self.entities.query()
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) -> Option<R> {
        self.resources.insert(resource)
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Result<R, NoSuchResource> {
        self.resources.remove().ok_or_else(|| NoSuchResource)
    }

    pub fn resource_entry<R: Resource>(&mut self) -> ResourceEntry<R> {
        self.resources.entry()
    }

    pub fn contains_resource<R: Resource>(&self) -> bool {
        self.resources.contains::<R>()
    }

    pub fn resource<R: Resource>(&self) -> Result<ResourceRef<R>, ResourceError> {
        self.resources.get()
    }

    pub fn resource_mut<R: Resource>(&self) -> Result<ResourceRefMut<R>, ResourceError> {
        self.resources.get_mut()
    }

    pub fn fetch<'a, RB>(&'a self) -> <RB::Effectors as Fetch<'a>>::Refs
    where
        RB: ResourceBundle,
        RB::Effectors: Fetch<'a>,
    {
        RB::effectors().fetch(self)
    }

    pub fn modification_queue(&self) -> ModificationQueue {
        self.modification_queues.get()
    }

    pub fn apply_range<R>(&mut self, queue: &mut ModificationQueue, range: R)
    where
        R: RangeBounds<usize>,
    {
        queue.drain(..).for_each(|closure| closure(self));
    }

    pub fn apply_all(&mut self, mut queue: ModificationQueue) {
        self.apply_range(&mut queue, ..);
    }

    pub(crate) fn write_archetypes<Q: Query>(&self, _archetypes: &mut ArchetypeSet) {
        println!("archetype handling is not implemented yet");
        //archetypes.extend(self.entities.query_scope::<Q>());
    }
}

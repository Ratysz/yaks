use fixedbitset::FixedBitSet;
use hecs::World as Entities;
use resources::Resources;

use crate::{
    executor::ExecutorId,
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeSet, Component, ComponentBundle, ComponentError, ComponentRef, ComponentRefMut,
    Components, DynamicComponentBundle, Entity, Executor, NoSuchEntity, NoSuchResource, Query,
    QueryBorrow, Resource, ResourceEntry, ResourceError, ResourceRef, ResourceRefMut,
};

#[derive(Default)]
pub struct World {
    entities: Entities,
    resources: Resources,
    executor_rebuild_tracking: FixedBitSet,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, components: impl DynamicComponentBundle) -> Entity {
        self.executor_rebuild_tracking.clear();
        self.entities.spawn(components)
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.entities.despawn(entity).map(|_| {
            self.executor_rebuild_tracking.clear();
        })
    }

    pub fn despawn_all(&mut self) {
        self.executor_rebuild_tracking.clear();
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
        self.entities.insert(entity, components).map(|_| {
            self.executor_rebuild_tracking.clear();
        })
    }

    pub fn remove_components<T: ComponentBundle>(
        &mut self,
        entity: Entity,
    ) -> Result<T, ComponentError> {
        self.entities.remove(entity).map(|result| {
            self.executor_rebuild_tracking.clear();
            result
        })
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

    pub fn fetch<'a, F>(&'a self) -> <F::Effectors as Fetch<'a>>::Refs
    where
        F: ResourceBundle,
        F::Effectors: Fetch<'a>,
    {
        F::effectors().fetch(self)
    }

    pub(crate) fn write_touched_archetypes<Q: Query>(&self, set: &mut ArchetypeSet) {
        set.extend(self.entities.query_scope::<Q>());
    }

    pub(crate) fn new_executor_id(&mut self) -> ExecutorId {
        let length = self.executor_rebuild_tracking.len();
        self.executor_rebuild_tracking.grow(length + 1);
        ExecutorId(length)
    }

    pub(crate) fn executor_needs_rebuilding(&self, id: ExecutorId) -> bool {
        !self.executor_rebuild_tracking[id.0]
    }

    pub(crate) fn executor_rebuilt(&mut self, id: ExecutorId) {
        self.executor_rebuild_tracking.set(id.0, true);
    }
}

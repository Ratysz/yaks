use std::ops::RangeBounds;

use crate::{
    Component, ComponentBundle, ComponentError, ComponentRef, ComponentRefMut,
    DynamicComponentBundle, Entity, ModificationQueue, NoSuchEntity, NoSuchResource, Query,
    QueryBorrow, Resource, World,
};

pub struct WorldProxy<'a> {
    pub(crate) world: &'a World,
    queue: &'a mut ModificationQueue,
}

impl<'a> WorldProxy<'a> {
    pub(crate) fn new(world: &'a World, queue: &'a mut ModificationQueue) -> Self {
        Self { world, queue }
    }

    pub fn spawn<C>(&mut self, components: C)
    where
        C: DynamicComponentBundle + Send + Sync + 'static,
    {
        self.queue.push(move |world| {
            world.spawn(components);
        });
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        if !self.is_alive(entity) {
            return Err(NoSuchEntity);
        }
        self.queue.push(move |world| {
            world.despawn(entity);
        });
        Ok(())
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world.is_alive(entity)
    }

    pub fn add_components<C>(&mut self, entity: Entity, components: C) -> Result<(), NoSuchEntity>
    where
        C: DynamicComponentBundle + Send + Sync + 'static,
    {
        if !self.is_alive(entity) {
            return Err(NoSuchEntity);
        }
        self.queue.push(move |world| {
            world.add_components(entity, components);
        });
        Ok(())
    }

    pub fn remove_components<C>(&mut self, entity: Entity) -> Result<(), ComponentError>
    where
        C: ComponentBundle,
    {
        // TODO ComponentError::MissingComponent
        if !self.is_alive(entity) {
            return Err(ComponentError::NoSuchEntity);
        }
        self.queue.push(move |world| {
            world.remove_components::<C>(entity);
        });
        Ok(())
    }

    pub fn component<C>(&self, entity: Entity) -> Result<ComponentRef<C>, ComponentError>
    where
        C: Component,
    {
        // TODO statically verify accessibility?
        self.world.component(entity)
    }

    pub fn component_mut<C>(&self, entity: Entity) -> Result<ComponentRefMut<C>, ComponentError>
    where
        C: Component,
    {
        // TODO statically verify accessibility?
        self.world.component_mut(entity)
    }

    /*pub fn components(&self, entity: Entity) -> Result<Components, NoSuchEntity> {
        self.entities.entity(entity)
    }*/

    pub fn add_resource<R>(&mut self, resource: R)
    where
        R: Resource,
    {
        self.queue.push(move |world| {
            world.add_resource(resource);
        });
    }

    pub fn remove_resource<R>(&mut self) -> Result<(), NoSuchResource>
    where
        R: Resource,
    {
        if !self.contains_resource::<R>() {
            return Err(NoSuchResource);
        }
        self.queue.push(move |world| {
            world.remove_resource::<R>();
        });
        Ok(())
    }

    pub fn contains_resource<R>(&self) -> bool
    where
        R: Resource,
    {
        self.world.contains_resource::<R>()
    }

    pub fn modification_queue(&self) -> ModificationQueue {
        self.world.modification_queue()
    }

    pub fn apply_range<R>(&mut self, queue: &mut ModificationQueue, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.queue.extend(queue.drain(range));
    }

    pub fn apply_all(&mut self, queue: ModificationQueue) {
        self.queue.merge(queue);
    }
}

use std::{
    any::{type_name, TypeId},
    ops::RangeBounds,
};

use crate::{
    error::{ComponentError, NoSuchEntity, NoSuchResource},
    query_bundle::QueryEffector,
    system::{ArchetypeSet, SystemBorrows},
    Component, ComponentBundle, ComponentRef, ComponentRefMut, DynamicComponentBundle, Entity,
    ModificationQueue, Query, QueryBorrow, Resource, World,
};

pub struct WorldProxy<'a> {
    pub(crate) world: &'a World,
    queue: &'a mut ModificationQueue,
    debug_id: &'a str,
    borrows: &'a SystemBorrows,
    archetypes: &'a ArchetypeSet,
}

impl<'a> WorldProxy<'a> {
    pub(crate) fn new(
        world: &'a World,
        queue: &'a mut ModificationQueue,
        debug_id: &'a str,
        borrows: &'a SystemBorrows,
        archetypes: &'a ArchetypeSet,
    ) -> Self {
        Self {
            world,
            queue,
            debug_id,
            borrows,
            archetypes,
        }
    }

    fn can_access<C>(&self) -> bool
    where
        C: Component,
    {
        self.borrows
            .components_immutable
            .contains(&TypeId::of::<C>())
            || self.can_mut_access::<C>()
    }

    fn can_mut_access<C>(&self) -> bool
    where
        C: Component,
    {
        self.borrows.components_mutable.contains(&TypeId::of::<C>())
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
        if !self.can_access::<C>() {
            panic!(
                "system '{0}' can't access {1} of Entity({2:?}) immutably: \
                 it's not required by the queries, \
                 or declared in `SystemBuilder::component()`",
                self.debug_id,
                type_name::<C>(),
                entity,
            );
        }
        self.world.component(entity)
    }

    pub fn component_mut<C>(&self, entity: Entity) -> Result<ComponentRefMut<C>, ComponentError>
    where
        C: Component,
    {
        if !self.can_access::<C>() {
            panic!(
                "system '{0}' can't access {1} of Entity({2:?}) mutably: \
                 it's not required by the queries, \
                 or declared in `SystemBuilder::component()`",
                self.debug_id,
                type_name::<C>(),
                entity,
            );
        }
        self.world.component_mut(entity)
    }

    /*pub fn components(&self, entity: Entity) -> Result<Components, NoSuchEntity> {
        self.entities.entity(entity)
    }*/

    pub fn query<Q>(&self, _: QueryEffector<Q>) -> QueryBorrow<Q>
    where
        Q: Query + Send + Sync,
    {
        self.world.query()
    }

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
        self.queue.absorb(queue);
    }
}

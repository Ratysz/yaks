use crate::{
    Component, ComponentError, ComponentRef, ComponentRefMut, DynamicComponentBundle, Entity,
    NoSuchEntity, Query, QueryBorrow, World,
};

pub struct WorldProxy<'a> {
    pub(crate) world: &'a World,
}

impl<'a> WorldProxy<'a> {
    pub(crate) fn new(world: &'a World) -> Self {
        Self { world }
    }

    /*pub fn later(&self) -> &mut L8r<World> {
        &self.later.get_mut()
    }*/

    /*pub fn spawn(&mut self, components: impl DynamicComponentBundle + Send + Sync + 'static) {
        self.later.spawn(components);
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        if !self.entities.contains(entity) {
            return Err(NoSuchEntity);
        }
        self.later.despawn(entity);
        Ok(())
    }*/

    /*pub fn despawn_all(&mut self) {
        self.later.clear();
    }*/

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world.is_alive(entity)
    }

    pub fn component<C>(&self, entity: Entity) -> Result<ComponentRef<C>, ComponentError>
    where
        C: Component,
    {
        self.world.component(entity)
    }

    pub fn component_mut<C>(&self, entity: Entity) -> Result<ComponentRefMut<C>, ComponentError>
    where
        C: Component,
    {
        self.world.component_mut(entity)
    }
}

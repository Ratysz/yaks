use hecs::{Entity, NoSuchEntity, Query, QueryBorrow, QueryOne, World};

use crate::{QueryMarker, SystemId};

pub struct SystemContext<'scope> {
    pub(crate) system_id: Option<SystemId>,
    pub(crate) world: &'scope World,
}

impl<'scope> SystemContext<'scope> {
    pub fn new(world: &'scope World) -> Self {
        Self {
            system_id: None,
            world,
        }
    }

    pub fn id(&self) -> Option<SystemId> {
        self.system_id
    }

    pub fn query<Q>(&self, _: QueryMarker<Q>) -> QueryBorrow<'_, Q>
    where
        Q: Query + Send + Sync,
    {
        self.world.query()
    }

    pub fn query_one<Q>(
        &self,
        _: QueryMarker<Q>,
        entity: Entity,
    ) -> Result<QueryOne<'_, Q>, NoSuchEntity>
    where
        Q: Query + Send + Sync,
    {
        self.world.query_one(entity)
    }
}

impl<'scope> From<&'scope World> for SystemContext<'scope> {
    fn from(world: &'scope World) -> Self {
        Self {
            system_id: None,
            world,
        }
    }
}

impl<'scope> From<&'scope mut World> for SystemContext<'scope> {
    fn from(world: &'scope mut World) -> Self {
        Self {
            system_id: None,
            world,
        }
    }
}

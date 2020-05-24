use hecs::{Entity, NoSuchEntity, Query, QueryBorrow, QueryOne, World};

use crate::{QueryMarker, SystemId};

pub struct SystemContext<'scope> {
    pub system_id: SystemId,
    pub(crate) world: &'scope World,
}

impl<'scope> SystemContext<'scope> {
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

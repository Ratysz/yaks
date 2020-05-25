use hecs::{Entity, Fetch, NoSuchEntity, Query, QueryBorrow, QueryOne, World};

use crate::{batch, QueryMarker, SystemId};

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

    pub fn batch<'query, 'world, Q, F>(
        &self,
        query_borrow: &'query mut QueryBorrow<'world, Q>,
        batch_size: u32,
        for_each: F,
    ) where
        Q: Query + Send + Sync + 'query,
        F: Fn(Entity, <<Q as Query>::Fetch as Fetch<'query>>::Item) + Send + Sync,
    {
        batch(query_borrow, batch_size, for_each);
    }
}

use hecs::{Entity, Fetch, NoSuchEntity, Query, QueryBorrow, QueryOne, World};

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

    pub fn batch<'query, 'world, Q, F>(
        &self,
        query_borrow: &'query mut QueryBorrow<'world, Q>,
        _batch_size: u32,
        for_each: F,
    ) where
        Q: Query + Send + Sync + 'query,
        F: Fn(Entity, <<Q as Query>::Fetch as Fetch<'query>>::Item) + Send + Sync,
    {
        #[cfg(feature = "parallel")]
        {
            let iterator = query_borrow.iter_batched(_batch_size);
            // Due to how rayon works, this will automatically run on either the global
            // or a local thread pool, depending on in scope of which batch() is called.
            rayon::scope(|scope| {
                iterator.for_each(|batch| {
                    scope.spawn(|_| {
                        batch.for_each(|(entity, components)| for_each(entity, components));
                    });
                });
            });
        }
        #[cfg(not(feature = "parallel"))]
        {
            query_borrow
                .iter()
                .for_each(|(entity, components)| for_each(entity, components));
        }
    }
}

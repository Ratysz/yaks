use hecs::{Query, QueryBorrow, World};
use resources::Resources;

use crate::{query_bundle::QueryEffector, ModQueue, ModQueuePool};

pub struct WorldFacade<'a> {
    pub world: &'a World,
    pub resources: &'a Resources,
    mod_queues: &'a ModQueuePool,
}

impl<'a> WorldFacade<'a> {
    pub(crate) fn new(
        world: &'a World,
        resources: &'a Resources,
        mod_queues: &'a ModQueuePool,
    ) -> Self {
        Self {
            world,
            resources,
            mod_queues,
        }
    }

    pub fn new_mod_queue(&self) -> ModQueue {
        self.mod_queues.new_mod_queue()
    }

    pub fn query<Q>(&self, _: QueryEffector<Q>) -> QueryBorrow<Q>
    where
        Q: Query + Send + Sync,
    {
        self.world.query()
    }
}

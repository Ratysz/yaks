use hecs::{Query, QueryBorrow, World};
use resources::Resources;

use crate::{query_bundle::QueryEffector, ModQueue, ModQueuePool};

#[cfg(feature = "parallel")]
use crate::Scope;

pub struct SystemContext<'scope> {
    pub world: &'scope World,
    pub resources: &'scope Resources,
    mod_queues: &'scope ModQueuePool,
    #[cfg(feature = "parallel")]
    scope: Option<&'scope Scope<'scope>>,
}

impl<'scope> SystemContext<'scope> {
    pub(crate) fn new(
        world: &'scope World,
        resources: &'scope Resources,
        mod_queues: &'scope ModQueuePool,
        #[cfg(feature = "parallel")] scope: Option<&'scope Scope<'scope>>,
    ) -> Self {
        Self {
            world,
            resources,
            mod_queues,
            #[cfg(feature = "parallel")]
            scope,
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

    #[cfg(feature = "parallel")]
    pub fn scope(&self) -> Option<Scope> {
        self.scope.map(|scope| scope.scope())
    }
}

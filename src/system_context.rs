use hecs::{Entity, Fetch, Query, QueryBorrow, World};
use resources::Resources;

use crate::{
    fetch_components::{Fetch as ComponentFetch, FetchComponents as _},
    query_bundle::{QueryEffector, QuerySingle},
    ModQueue, ModQueuePool,
};

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

    pub fn fetch<Q>(
        &self,
        _: QueryEffector<Q>,
        entity: Entity,
    ) -> <Q::ComponentEffectors as ComponentFetch>::Refs
    where
        Q: Query + QuerySingle + Send + Sync,
        for<'a> Q::ComponentEffectors: ComponentFetch<'a>,
    {
        self.world.fetch::<Q>(entity)
    }

    #[cfg(feature = "parallel")]
    pub fn scope(&self) -> Option<Scope> {
        self.scope.map(|scope| scope.scope())
    }

    #[cfg(feature = "parallel")]
    pub fn batch<'q, 'w, F, Q>(
        &self,
        query_borrow: &'q mut QueryBorrow<'w, Q>,
        batch_size: u32,
        closure: F,
    ) where
        F: Fn((Entity, <<Q as Query>::Fetch as Fetch<'q>>::Item)) + Send + Sync,
        Q: Query + Send + Sync + 'q,
    {
        if let Some(scope) = self.scope {
            scope.scope().batch(query_borrow, batch_size, closure);
        } else {
            query_borrow.iter().for_each(|item| closure(item));
        }
    }

    #[cfg(not(feature = "parallel"))]
    pub fn batch<'q, 'w, F, Q>(&self, query_borrow: &'q mut QueryBorrow<'w, Q>, _: u32, closure: F)
    where
        F: Fn((Entity, <<Q as Query>::Fetch as Fetch<'q>>::Item)) + Send + Sync,
        Q: Query + Send + Sync + 'q,
    {
        query_borrow.iter().for_each(|item| closure(item));
    }
}

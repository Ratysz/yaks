use std::marker::PhantomData;

use crate::{Component, Query, QueryIter, World};

pub struct QueryEffector<'a, Q>
where
    Q: Query<'a> + Send + Sync,
{
    phantom_data: PhantomData<(&'a (), Q)>,
}

impl<'a, Q> QueryEffector<'a, Q>
where
    Q: Query<'a> + Send + Sync,
{
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }

    pub fn query(&self, world: &'a World) -> QueryIter<'a, Q> {
        world.query()
    }
}

pub trait QueryBundle<'a>: Send + Sync {
    type QueryEffectors;

    fn query_effectors() -> Self::QueryEffectors;
}

impl<'a, C> QueryBundle<'a> for &'a C
where
    C: Component,
{
    type QueryEffectors = QueryEffector<'a, Self>;

    fn query_effectors() -> Self::QueryEffectors {
        QueryEffector::new()
    }
}

impl<'a, C> QueryBundle<'a> for &'a mut C
where
    C: Component,
{
    type QueryEffectors = QueryEffector<'a, Self>;

    fn query_effectors() -> Self::QueryEffectors {
        QueryEffector::new()
    }
}

impl<'a, Q> QueryBundle<'a> for Option<Q>
where
    Q: Query<'a> + Send + Sync,
{
    type QueryEffectors = QueryEffector<'a, Self>;

    fn query_effectors() -> Self::QueryEffectors {
        QueryEffector::new()
    }
}

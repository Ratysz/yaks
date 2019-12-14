use std::marker::PhantomData;

use crate::{
    query_bundle::QueryBundle, resource_bundle::ResourceBundle, Component, Query, QueryIter, World,
};

trait SystemFunction<'a, R, Q>
where
    R: ResourceBundle<'a>,
    Q: QueryBundle<'a>,
{
}

pub struct System<'a, R, Q, F>
where
    R: ResourceBundle<'a>,
    Q: QueryBundle<'a>,
    F: FnMut(&World, R::Refs, Q::QueryEffectors),
{
    phantom_data: PhantomData<(&'a (), R, Q)>,
    function: F,
}

impl<'a, R, Q, F> System<'a, R, Q, F>
where
    R: ResourceBundle<'a>,
    Q: QueryBundle<'a>,
    F: FnMut(&World, R::Refs, Q::QueryEffectors),
{
    pub fn new(function: F) -> Self {
        Self {
            phantom_data: PhantomData,
            function,
        }
    }

    pub fn run(&mut self, world: &'a World) {
        (self.function)(world, R::fetch(world), Q::query_effectors())
    }
}

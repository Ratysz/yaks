use super::{ResourceTuple, WrappableTuple};
use crate::Executor;

// TODO consider exposing.

/// Specifies how a tuple of references may be extracted from the implementor and used
/// as resources when running an executor.
pub trait Wrap<Source, Marker>: ResourceTuple + Sized {
    fn wrap_and_run(executor: &mut Executor<Self>, world: &hecs::World, resources: Source);
}

impl<Source, Marker, W> Wrap<Source, Marker> for W
where
    W: ResourceTuple
        + WrappableTuple<
            Source,
            Marker,
            Wrapped = <W as ResourceTuple>::Wrapped,
            BorrowTuple = <W as ResourceTuple>::BorrowTuple,
        >,
{
    fn wrap_and_run(executor: &mut Executor<Self>, world: &hecs::World, resources: Source) {
        let mut fetched = W::fetch(resources);
        let wrapped = W::wrap(&mut fetched, &mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

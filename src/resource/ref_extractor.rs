use hecs::World;

use super::{ResourceTuple, Wrappable};
use crate::Executor;

// TODO consider exposing.

/// Specifies how a tuple of references may be extracted from the implementor and used
/// as resources when running an executor.
pub trait RefExtractor<RefSource>: ResourceTuple + Sized {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: RefSource);
}

impl<W, T> RefExtractor<W> for T
where
    W: Wrappable<Wrapped = T::Wrapped, BorrowTuple = T::BorrowTuple>,
    T: ResourceTuple,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: W) {
        let wrapped = resources.wrap(&mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

/*impl RefExtractor<()> for () {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, _: ()) {
        executor.inner.run(world, ());
    }
}

impl<R0> RefExtractor<&R0> for (Ref<R0>,)
where
    Self: ResourceTuple,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, mut resources: &R0) {
        let wrapped = resources.wrap(&mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

impl<R0> RefExtractor<&mut R0> for (Mut<R0>,)
where
    Self: ResourceTuple,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, mut resources: &mut R0) {
        let wrapped = resources.wrap(&mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

macro_rules! impl_ref_extractor {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> RefExtractor<($(&mut $letter,)*)> for ($($letter,)*)
        where
            Self: ResourceTuple,
            $($letter: Send + Sync,)*
        {
            fn extract_and_run(
                executor: &mut Executor<Self>,
                world: &World,
                mut resources: ($(&mut $letter,)*),
            ) {
                let wrapped = resources.wrap(&mut executor.borrows);
                executor.inner.run(world, wrapped);
            }
        }
    }
}

impl_for_tuples!(impl_ref_extractor);*/

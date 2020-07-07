use hecs::World;

use super::{ResourceTuple, ResourceWrap};
use crate::Executor;

// TODO consider exposing.

/// Specifies how a tuple of references may be extracted from the implementor and used
/// as resources when running an executor.
pub trait RefExtractor<RefSource>: ResourceTuple + Sized {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: RefSource);
}

impl RefExtractor<()> for () {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, _: ()) {
        executor.inner.run(world, ());
    }
}

impl<R0> RefExtractor<&mut R0> for (R0,)
where
    R0: Send + Sync,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, mut resources: &mut R0) {
        let wrapped = resources.wrap(&mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

impl<R0> RefExtractor<(&mut R0,)> for (R0,)
where
    R0: Send + Sync,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, mut resources: (&mut R0,)) {
        let wrapped = resources.wrap(&mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

macro_rules! impl_ref_extractor {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> RefExtractor<($(&mut $letter,)*)> for ($($letter,)*)
        where
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

impl_for_tuples!(impl_ref_extractor);

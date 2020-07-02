use hecs::World;

use crate::{AtomicBorrow, Executor, ResourceCell};

pub trait ResourceTuple {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;
    const LENGTH: usize;

    fn instantiate_borrows() -> Self::BorrowTuple;
}

pub trait ResourceWrap {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

pub trait RefExtractor<RefSource>: ResourceTuple + Sized {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: RefSource);
}

impl ResourceTuple for () {
    type Wrapped = ();
    type BorrowTuple = ();
    const LENGTH: usize = 0;

    fn instantiate_borrows() -> Self::BorrowTuple {}
}

impl ResourceWrap for () {
    type Wrapped = ();
    type BorrowTuple = ();

    fn wrap(&mut self, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
}

impl RefExtractor<()> for () {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, _: ()) {
        executor.inner.run(world, ());
    }
}

impl<R0> ResourceTuple for (R0,)
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);
    const LENGTH: usize = 1;

    fn instantiate_borrows() -> Self::BorrowTuple {
        (AtomicBorrow::new(),)
    }
}

impl<R0> ResourceWrap for &'_ mut R0
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceCell::new(self, &mut borrows.0),)
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

impl<R0> ResourceWrap for (&'_ mut R0,)
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceCell::new(self.0, &mut borrows.0),)
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

macro_rules! swap_to_atomic_borrow {
    ($anything:tt) => {
        AtomicBorrow
    };
    (new $anything:tt) => {
        AtomicBorrow::new()
    };
}

macro_rules! impl_resource_tuple {
    ($($letter:ident),*) => {
        impl<$($letter),*> ResourceTuple for ($($letter,)*)
        where
            $($letter: Send + Sync,)*
        {
            type Wrapped = ($(ResourceCell<$letter>,)*);
            type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);
            const LENGTH: usize = count!($($letter)*);

            fn instantiate_borrows() -> Self::BorrowTuple {
                ($(swap_to_atomic_borrow!(new $letter),)*)
            }
        }
    }
}

impl_for_tuples!(impl_resource_tuple);

macro_rules! impl_resource_wrap {
    ($($letter:ident),*) => {
        paste::item! {
            impl<$($letter),*> ResourceWrap for ($(&'_ mut $letter,)*)
            where
                $($letter: Send + Sync,)*
            {
                type Wrapped = ($(ResourceCell<$letter>,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                #[allow(non_snake_case)]
                fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
                    let ($([<S $letter>],)*) = self;
                    let ($([<B $letter>],)*) = borrows;
                    ($( ResourceCell::new([<S $letter>], [<B $letter>]) ,)*)
                }
            }
        }
    }
}

impl_for_tuples!(impl_resource_wrap);

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

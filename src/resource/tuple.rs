use std::marker::PhantomData;

use super::{AtomicBorrow, ResourceMutCell, ResourceRefCell};

/// Marker to denote that the executor will be borrowing the type `Resource` immutably.
pub struct Ref<Resource>(PhantomData<Resource>);

/// Marker to denote that the executor will be borrowing the type `Resource` mutably.
pub struct Mut<Resource>(PhantomData<Resource>);

pub trait ResourceSingle {
    type Wrapped: Send + Sync;
}

impl<R0> ResourceSingle for Ref<R0>
where
    R0: Send + Sync,
{
    type Wrapped = ResourceRefCell<R0>;
}

impl<R0> ResourceSingle for Mut<R0>
where
    R0: Send + Sync,
{
    type Wrapped = ResourceMutCell<R0>;
}

pub trait ResourceTuple {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;
    const LENGTH: usize;

    fn instantiate_borrows() -> Self::BorrowTuple;
}

impl<R0> ResourceTuple for Ref<R0>
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceRefCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);
    const LENGTH: usize = 1;

    fn instantiate_borrows() -> Self::BorrowTuple {
        (AtomicBorrow::new(),)
    }
}

impl<R0> ResourceTuple for Mut<R0>
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceMutCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);
    const LENGTH: usize = 1;

    fn instantiate_borrows() -> Self::BorrowTuple {
        (AtomicBorrow::new(),)
    }
}

impl ResourceTuple for () {
    type Wrapped = ();
    type BorrowTuple = ();
    const LENGTH: usize = 0;

    fn instantiate_borrows() -> Self::BorrowTuple {}
}

impl<R0> ResourceTuple for (R0,)
where
    R0: ResourceSingle,
{
    type Wrapped = (R0::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);
    const LENGTH: usize = 1;

    fn instantiate_borrows() -> Self::BorrowTuple {
        (AtomicBorrow::new(),)
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
            $($letter: ResourceSingle,)*
        {
            type Wrapped = ($($letter::Wrapped,)*);
            type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);
            const LENGTH: usize = count!($($letter)*);

            fn instantiate_borrows() -> Self::BorrowTuple {
                ($(swap_to_atomic_borrow!(new $letter),)*)
            }
        }
    }
}

impl_for_tuples!(impl_resource_tuple, no_single);

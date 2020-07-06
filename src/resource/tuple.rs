use super::{AtomicBorrow, ResourceCell};

/// Specifies how a tuple behaves when used as the generic parameter of an executor.
pub trait ResourceTuple {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;
    const LENGTH: usize;

    fn instantiate_borrows() -> Self::BorrowTuple;
}

impl ResourceTuple for () {
    type Wrapped = ();
    type BorrowTuple = ();
    const LENGTH: usize = 0;

    fn instantiate_borrows() -> Self::BorrowTuple {}
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

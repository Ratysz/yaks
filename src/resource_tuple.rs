use crate::{AtomicBorrow, ResourceCell};

pub trait ResourceTuple {
    type Wrapped;
    type BorrowTuple;
    const LENGTH: usize;

    fn instantiate_borrows() -> Self::BorrowTuple;
}

pub trait ResourceWrap {
    type Types;
    type Wrapped;
    type BorrowTuple;

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl<'a, R0> ResourceWrap for &'a mut R0
where
    R0: Send + Sync,
{
    type Types = (R0,);
    type Wrapped = (ResourceCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceCell::new(self, &mut borrows.0),)
    }
}

impl ResourceTuple for () {
    type Wrapped = ();
    type BorrowTuple = ();
    const LENGTH: usize = 0;

    fn instantiate_borrows() -> Self::BorrowTuple {}
}

impl ResourceWrap for () {
    type Types = ();
    type Wrapped = ();
    type BorrowTuple = ();

    fn wrap(&mut self, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
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

impl<R0> ResourceWrap for (&'_ mut R0,)
where
    R0: Send + Sync,
{
    type Types = (R0,);
    type Wrapped = (ResourceCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceCell::new(self.0, &mut borrows.0),)
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
                type Types = ($($letter,)*);
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

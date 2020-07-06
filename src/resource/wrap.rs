use super::{AtomicBorrow, ResourceCell};

/// Specifies how a tuple of references is wrapped into a tuple of cells.
pub trait ResourceWrap {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;

    fn wrap(&mut self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl ResourceWrap for () {
    type Wrapped = ();
    type BorrowTuple = ();

    fn wrap(&mut self, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
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

macro_rules! swap_to_atomic_borrow {
    ($anything:tt) => {
        AtomicBorrow
    };
    (new $anything:tt) => {
        AtomicBorrow::new()
    };
}

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

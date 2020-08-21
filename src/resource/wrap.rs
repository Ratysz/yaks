use super::{AtomicBorrow, ResourceMutCell, ResourceRefCell};

pub trait WrappableSingle {
    type Wrapped: Send + Sync;

    fn wrap(self, borrow: &mut AtomicBorrow) -> Self::Wrapped;
}

impl<R0> WrappableSingle for &'_ R0
where
    R0: Send + Sync,
{
    type Wrapped = ResourceRefCell<R0>;

    fn wrap(self, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(self, borrow)
    }
}

impl<R0> WrappableSingle for &'_ mut R0
where
    R0: Send + Sync,
{
    type Wrapped = ResourceMutCell<R0>;

    fn wrap(self, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(self, borrow)
    }
}

pub trait Wrappable {
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;

    fn wrap(self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl<R0> Wrappable for &'_ R0
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceRefCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceRefCell::new(self, &mut borrows.0),)
    }
}

impl<R0> Wrappable for &'_ mut R0
where
    R0: Send + Sync,
{
    type Wrapped = (ResourceMutCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceMutCell::new(self, &mut borrows.0),)
    }
}

impl Wrappable for () {
    type Wrapped = ();
    type BorrowTuple = ();

    fn wrap(self, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
}

impl<R0> Wrappable for (R0,)
where
    R0: WrappableSingle,
{
    type Wrapped = (R0::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn wrap(self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (self.0.wrap(&mut borrows.0),)
    }
}

macro_rules! swap_to_atomic_borrow {
    ($anything:tt) => {
        AtomicBorrow
    };
}

macro_rules! impl_wrappable {
    ($($letter:ident),*) => {
        paste::item! {
            impl<$($letter),*> Wrappable for ($($letter,)*)
            where
                $($letter: WrappableSingle,)*
            {
                type Wrapped = ($($letter::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                #[allow(non_snake_case)]
                fn wrap(self, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
                    let ($([<S $letter>],)*) = self;
                    let ($([<B $letter>],)*) = borrows;
                    ($([<S $letter>].wrap([<B $letter>]),)*)
                }
            }
        }
    }
}

impl_for_tuples!(impl_wrappable);

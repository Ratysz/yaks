use std::ops::{Deref, DerefMut};

use super::{AtomicBorrow, Mut, Ref, ResourceMutCell, ResourceRefCell};

/// Describes which (and how) intermediate type can be obtained from `Source`;
///
/// `Source` will need some mechanism for inner mutability for this trait to be implementable.
///
/// Implement on `Ref<T>` and `Mut<T>` to enable using `Source` as the resources argument in
/// `Executor::run()` and `System::run()`.
pub trait MarkerGet<Source: Copy> {
    /// The intermediate type returned by `fetch()`.
    /// Must be `Deref<T>` when implemented on `Ref<T>` and `DerefMut<T>` on `Mut<T>`.
    type Fetched;

    /// Retrieves the intermediate type `Fetch` from `Source`.
    fn fetch(source: Source) -> Self::Fetched;
}

pub trait WrappableSingle<Source, Marker> {
    type Fetched;
    type Wrapped: Send + Sync;

    fn fetch(source: Source) -> Self::Fetched;

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped;
}

impl<'a, Source, R> WrappableSingle<Source, Source> for Ref<R>
where
    Self: MarkerGet<Source>,
    <Self as MarkerGet<Source>>::Fetched: Deref<Target = R>,
    Source: Copy,
    R: Send + Sync,
{
    type Fetched = <Self as MarkerGet<Source>>::Fetched;
    type Wrapped = ResourceRefCell<R>;

    fn fetch(source: Source) -> Self::Fetched {
        <Self as MarkerGet<Source>>::fetch(source)
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, Source, R> WrappableSingle<Source, Source> for Mut<R>
where
    Self: MarkerGet<Source>,
    <Self as MarkerGet<Source>>::Fetched: DerefMut<Target = R>,
    Source: Copy,
    R: Send + Sync,
{
    type Fetched = <Self as MarkerGet<Source>>::Fetched;
    type Wrapped = ResourceMutCell<R>;

    fn fetch(source: Source) -> Self::Fetched {
        <Self as MarkerGet<Source>>::fetch(source)
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<&'a R, ()> for Ref<R>
where
    R: Send + Sync,
{
    type Fetched = &'a R;
    type Wrapped = ResourceRefCell<R>;

    fn fetch(source: &'a R) -> Self::Fetched {
        source
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<&'a mut R, ()> for Mut<R>
where
    R: Send + Sync,
{
    type Fetched = &'a mut R;
    type Wrapped = ResourceMutCell<R>;

    fn fetch(source: &'a mut R) -> Self::Fetched {
        source
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<(&'a R,), ()> for Ref<R>
where
    R: Send + Sync,
{
    type Fetched = &'a R;
    type Wrapped = ResourceRefCell<R>;

    fn fetch(source: (&'a R,)) -> Self::Fetched {
        source.0
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<(&'a mut R,), ()> for Mut<R>
where
    R: Send + Sync,
{
    type Fetched = &'a mut R;
    type Wrapped = ResourceMutCell<R>;

    fn fetch(source: (&'a mut R,)) -> Self::Fetched {
        source.0
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

pub trait WrappableTuple<Source, Marker> {
    type Fetched;
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;

    fn fetch(source: Source) -> Self::Fetched;

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl<'a, Source, Marker, R> WrappableTuple<Source, Marker> for Ref<R>
where
    Self: WrappableSingle<Source, Marker>,
    R: Send + Sync,
{
    type Fetched = <Self as WrappableSingle<Source, Marker>>::Fetched;
    type Wrapped = (<Self as WrappableSingle<Source, Marker>>::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn fetch(source: Source) -> Self::Fetched {
        <Self as WrappableSingle<Source, Marker>>::fetch(source)
    }

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (<Self as WrappableSingle<Source, Marker>>::wrap(
            fetched,
            &mut borrows.0,
        ),)
    }
}

impl<'a, Source, Marker, R> WrappableTuple<Source, Marker> for Mut<R>
where
    Self: WrappableSingle<Source, Marker>,
    R: Send + Sync,
{
    type Fetched = <Self as WrappableSingle<Source, Marker>>::Fetched;
    type Wrapped = (<Self as WrappableSingle<Source, Marker>>::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn fetch(source: Source) -> Self::Fetched {
        <Self as WrappableSingle<Source, Marker>>::fetch(source)
    }

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (<Self as WrappableSingle<Source, Marker>>::wrap(
            fetched,
            &mut borrows.0,
        ),)
    }
}

impl<Source> WrappableTuple<Source, ()> for () {
    type Fetched = ();
    type Wrapped = ();
    type BorrowTuple = ();

    fn fetch(_: Source) -> Self::Fetched {}

    fn wrap(_: &mut Self::Fetched, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
}

macro_rules! swap_to_atomic_borrow {
    ($anything:tt) => {
        AtomicBorrow
    };
    (new $anything:tt) => {
        AtomicBorrow::new()
    };
}

macro_rules! impl_wrappable {
    ($letter:ident) => {
        impl<Source, Marker, $letter> WrappableTuple<Source, Marker> for ($letter,)
        where
            $letter: WrappableSingle<Source, Marker>,
        {
            type Fetched = ($letter::Fetched,);
            type Wrapped = ($letter::Wrapped,);
            type BorrowTuple = (AtomicBorrow,);

            fn fetch(source: Source) -> Self::Fetched {
                ($letter::fetch(source),)
            }

            fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
                ($letter::wrap(&mut fetched.0, &mut borrows.0),)
            }
        }
    };
    ($($letter:ident),*) => {
        paste::item! {
            impl<Source, $($letter),*> WrappableTuple<Source, Source> for ($($letter,)*)
            where
                Source: Copy,
                $($letter: WrappableSingle<Source, Source>,)*
            {
                type Fetched = ($($letter::Fetched,)*);
                type Wrapped = ($($letter::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                fn fetch(source: Source) -> Self::Fetched {
                    ($($letter::fetch(source),)*)
                }

                #[allow(non_snake_case)]
                fn wrap(
                    fetched: &mut Self::Fetched,
                    borrows: &mut Self::BorrowTuple
                ) -> Self::Wrapped {
                    let ($([<s_ $letter>],)*) = fetched;
                    let ($([<b_ $letter>],)*) = borrows;
                    ($($letter::wrap([<s_ $letter>], [<b_ $letter>]),)*)
                }
            }

            impl<$($letter,)* $([<W $letter>],)*> WrappableTuple<($($letter,)*), ()>
                for ($([<W $letter>],)*)
            where
                $([<W $letter>]: WrappableSingle<$letter, ()>,)*
            {
                type Fetched = ($([<W $letter>]::Fetched,)*);
                type Wrapped = ($([<W $letter>]::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                #[allow(non_snake_case)]
                fn fetch(source: ($($letter,)*)) -> Self::Fetched {
                    let ($([<s_ $letter>],)*) = source;
                    ($([<W $letter>]::fetch([<s_ $letter>]),)*)
                }

                #[allow(non_snake_case)]
                fn wrap(
                    fetched: &mut Self::Fetched,
                    borrows: &mut Self::BorrowTuple
                ) -> Self::Wrapped {
                    let ($([<s_ $letter>],)*) = fetched;
                    let ($([<b_ $letter>],)*) = borrows;
                    ($([<W $letter>]::wrap([<s_ $letter>], [<b_ $letter>]),)*)
                }
            }
        }
    }
}

impl_for_tuples!(impl_wrappable, all);

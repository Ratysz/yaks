use std::ops::{Deref, DerefMut};

use super::{AtomicBorrow, Mut, Ref, ResourceMutCell, ResourceRefCell};

/// Describes which (and how) intermediate type can be obtained from `Source`.
/// Implementing this trait is required **only** to enable using custom `Source` struct;
///
/// it is, effectively, already implemented for single types, all tuples up to 16, and,
/// with `resources-interop` feature, `resources::Resources`.
///
/// Implement on [`Ref<T>`](struct.Ref.html) and [`Mut<T>`](struct.Mut.html) to enable using
/// `Source` as the resources argument in [`Executor::run()`](struct.Executor.html#method.run)
/// and [`Run::run()`](trait.Run.html#method.run).
///
/// `Source` will need some mechanism for interior mutability for this trait to be
/// implementable on `Mut<T>`.
/// `Source` does not need to be `Send` or `Sync`, but `T` has to be both.
///
/// # Example:
/// ```rust
/// # use hecs::World;
/// use std::cell::{RefCell, RefMut};
/// use yaks::{Mut, Ref, Run};
///
/// struct CustomResources {
///     some_usize: RefCell<usize>,
///     some_f32: f32,
/// }
///
/// impl<'a> yaks::MarkerGet<&'a CustomResources> for yaks::Mut<usize> {
///     type Intermediate = RefMut<'a, usize>;
///
///     fn get(source: &'a CustomResources) -> Self::Intermediate {
///         source.some_usize.borrow_mut()
///     }
/// }
///
/// impl<'a> yaks::MarkerGet<&'a CustomResources> for yaks::Ref<f32> {
///     type Intermediate = &'a f32;
///
///     fn get(source: &'a CustomResources) -> Self::Intermediate {
///         &source.some_f32
///     }
/// }
///
/// fn system(some_f32: &f32, some_usize: &mut usize) {
///     *some_usize += *some_f32 as usize;
/// }
///
/// let world = World::new();
/// let resources = CustomResources {
///     some_usize: RefCell::new(0),
///     some_f32: 1.0,
/// };
///
/// let mut executor = yaks::Executor::<(Mut<usize>, Ref<f32>)>::builder()
///     .system(system)
///     .build();
/// executor.run(&world, &resources);
/// assert_eq!(*resources.some_usize.borrow(), 1);
///
/// system.run(&world, &resources);
/// assert_eq!(*resources.some_usize.borrow(), 2);
/// ```
pub trait MarkerGet<Source: Copy> {
    /// The intermediate type returned by `get()`.
    /// Must be `Deref<T>` when implemented on `Ref<T>` and `DerefMut<T>` on `Mut<T>`.
    type Intermediate;

    /// Retrieves the type `Intermediate` from `Source`.
    fn get(source: Source) -> Self::Intermediate;
}

pub trait WrappableSingle<Source, Marker> {
    type Intermediate;
    type Wrapped: Send + Sync;

    fn get(source: Source) -> Self::Intermediate;

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped;
}

impl<'a, Source, R> WrappableSingle<Source, Source> for Ref<R>
where
    Self: MarkerGet<Source>,
    <Self as MarkerGet<Source>>::Intermediate: Deref<Target = R>,
    Source: Copy,
    R: Send + Sync,
{
    type Intermediate = <Self as MarkerGet<Source>>::Intermediate;
    type Wrapped = ResourceRefCell<R>;

    fn get(source: Source) -> Self::Intermediate {
        <Self as MarkerGet<Source>>::get(source)
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, Source, R> WrappableSingle<Source, Source> for Mut<R>
where
    Self: MarkerGet<Source>,
    <Self as MarkerGet<Source>>::Intermediate: DerefMut<Target = R>,
    Source: Copy,
    R: Send + Sync,
{
    type Intermediate = <Self as MarkerGet<Source>>::Intermediate;
    type Wrapped = ResourceMutCell<R>;

    fn get(source: Source) -> Self::Intermediate {
        <Self as MarkerGet<Source>>::get(source)
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<&'a R, ()> for Ref<R>
where
    R: Send + Sync,
{
    type Intermediate = &'a R;
    type Wrapped = ResourceRefCell<R>;

    fn get(source: &'a R) -> Self::Intermediate {
        source
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<&'a mut R, ()> for Mut<R>
where
    R: Send + Sync,
{
    type Intermediate = &'a mut R;
    type Wrapped = ResourceMutCell<R>;

    fn get(source: &'a mut R) -> Self::Intermediate {
        source
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<(&'a R,), ()> for Ref<R>
where
    R: Send + Sync,
{
    type Intermediate = &'a R;
    type Wrapped = ResourceRefCell<R>;

    fn get(source: (&'a R,)) -> Self::Intermediate {
        source.0
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, R> WrappableSingle<(&'a mut R,), ()> for Mut<R>
where
    R: Send + Sync,
{
    type Intermediate = &'a mut R;
    type Wrapped = ResourceMutCell<R>;

    fn get(source: (&'a mut R,)) -> Self::Intermediate {
        source.0
    }

    fn wrap(fetched: &mut Self::Intermediate, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

pub trait WrappableTuple<Source, Marker> {
    type Intermediates;
    type Wrapped: Send + Sync;
    type BorrowTuple: Send + Sync;

    fn get(source: Source) -> Self::Intermediates;

    fn wrap(fetched: &mut Self::Intermediates, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl<'a, Source, Marker, R> WrappableTuple<Source, Marker> for Ref<R>
where
    Self: WrappableSingle<Source, Marker>,
    R: Send + Sync,
{
    type Intermediates = <Self as WrappableSingle<Source, Marker>>::Intermediate;
    type Wrapped = (<Self as WrappableSingle<Source, Marker>>::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn get(source: Source) -> Self::Intermediates {
        <Self as WrappableSingle<Source, Marker>>::get(source)
    }

    fn wrap(fetched: &mut Self::Intermediates, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
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
    type Intermediates = <Self as WrappableSingle<Source, Marker>>::Intermediate;
    type Wrapped = (<Self as WrappableSingle<Source, Marker>>::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn get(source: Source) -> Self::Intermediates {
        <Self as WrappableSingle<Source, Marker>>::get(source)
    }

    fn wrap(fetched: &mut Self::Intermediates, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (<Self as WrappableSingle<Source, Marker>>::wrap(
            fetched,
            &mut borrows.0,
        ),)
    }
}

impl<Source> WrappableTuple<Source, ()> for () {
    type Intermediates = ();
    type Wrapped = ();
    type BorrowTuple = ();

    fn get(_: Source) -> Self::Intermediates {}

    fn wrap(_: &mut Self::Intermediates, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
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
    () => {};
    ($letter:ident) => {
        paste::item! {
            impl<Source, Marker, [<Wrappable $letter>]> WrappableTuple<Source, Marker>
                for ([<Wrappable $letter>],)
            where
                [<Wrappable $letter>]: WrappableSingle<Source, Marker>,
            {
                type Intermediates = ([<Wrappable $letter>]::Intermediate,);
                type Wrapped = ([<Wrappable $letter>]::Wrapped,);
                type BorrowTuple = (AtomicBorrow,);

                fn get(source: Source) -> Self::Intermediates {
                    ([<Wrappable $letter>]::get(source),)
                }

                fn wrap(
                    ([<intermediate_ $letter:lower>], ): &mut Self::Intermediates,
                    ([<borrow_ $letter:lower>], ): &mut Self::BorrowTuple,
                ) -> Self::Wrapped {
                    ([<Wrappable $letter>]::wrap(
                        [<intermediate_ $letter:lower>],
                        [<borrow_ $letter:lower>],
                    ),)
                }
            }
        }
    };
    ($($letter:ident),*) => {
        paste::item! {
            impl<Source, $([<Wrappable $letter>],)*> WrappableTuple<Source, Source>
                for ($([<Wrappable $letter>],)*)
            where
                Source: Copy,
                $([<Wrappable $letter>]: WrappableSingle<Source, Source>,)*
            {
                type Intermediates = ($([<Wrappable $letter>]::Intermediate,)*);
                type Wrapped = ($([<Wrappable $letter>]::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                fn get(source: Source) -> Self::Intermediates {
                    ($([<Wrappable $letter>]::get(source),)*)
                }

                fn wrap(
                    ($([<intermediate_ $letter:lower>],)*): &mut Self::Intermediates,
                    ($([<borrow_ $letter:lower>],)*): &mut Self::BorrowTuple
                ) -> Self::Wrapped {
                    ($([<Wrappable $letter>]::wrap(
                        [<intermediate_ $letter:lower>],
                        [<borrow_ $letter:lower>],
                    ),)*)
                }
            }

            impl<$([<Source $letter>],)* $([<Wrappable $letter>],)*>
                WrappableTuple<($([<Source $letter>],)*), ()>
                for ($([<Wrappable $letter>],)*)
            where
                $([<Wrappable $letter>]: WrappableSingle<[<Source $letter>], ()>,)*
            {
                type Intermediates = ($([<Wrappable $letter>]::Intermediate,)*);
                type Wrapped = ($([<Wrappable $letter>]::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                fn get(
                    ($([<source_ $letter:lower>],)*): ($([<Source $letter>],)*)
                ) -> Self::Intermediates {
                    ($([<Wrappable $letter>]::get([<source_ $letter:lower>]),)*)
                }

                fn wrap(
                    ($([<intermediate_ $letter:lower>],)*): &mut Self::Intermediates,
                    ($([<borrow_ $letter:lower>],)*): &mut Self::BorrowTuple
                ) -> Self::Wrapped {
                    ($([<Wrappable $letter>]::wrap(
                        [<intermediate_ $letter:lower>],
                        [<borrow_ $letter:lower>],
                    ),)*)
                }
            }
        }
    }
}

impl_for_tuples!(impl_wrappable);

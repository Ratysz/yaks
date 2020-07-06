use super::Contains;

#[cfg(feature = "parallel")]
use crate::BorrowSet;

/// Specifies how a tuple of types may be borrowed from a tuple of cells.
pub trait Fetch<'a, T, M0>: Sized {
    fn fetch(resources: &'a T) -> Self;

    unsafe fn release(resources: &'a T);

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet);
}

impl<'a, T, M0, R0> Fetch<'a, T, M0> for &'a R0
where
    T: Contains<R0, M0>,
    R0: 'a,
{
    fn fetch(resources: &'a T) -> Self {
        T::borrow(resources)
    }

    unsafe fn release(resources: &'a T) {
        T::release(resources);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet) {
        T::set_resource_bit(&mut resource_set.immutable);
    }
}

impl<'a, T, M0, R0> Fetch<'a, T, M0> for &'a mut R0
where
    T: Contains<R0, M0>,
    R0: 'a,
{
    fn fetch(resources: &'a T) -> Self {
        T::borrow_mut(resources)
    }

    unsafe fn release(resources: &'a T) {
        T::release_mut(resources);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet) {
        T::set_resource_bit(&mut resource_set.mutable);
    }
}

impl<'a, T> Fetch<'a, T, ()> for () {
    fn fetch(_: &'a T) -> Self {}

    unsafe fn release(_: &'a T) {}

    #[cfg(feature = "parallel")]
    fn set_resource_bits(_: &mut BorrowSet) {}
}

impl<'a, T, M0, F0> Fetch<'a, T, (M0,)> for (F0,)
where
    F0: Fetch<'a, T, M0>,
{
    fn fetch(resources: &'a T) -> Self {
        (F0::fetch(resources),)
    }

    unsafe fn release(resources: &'a T) {
        F0::release(resources);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet) {
        F0::set_resource_bits(resource_set);
    }
}

macro_rules! impl_fetch {
    ($($letter:ident),*) => {
        paste::item! {
            impl<'a, T, $([<M $letter>],)* $([<F $letter>],)*> Fetch<'a, T, ($([<M $letter>],)*)>
                for ($([<F $letter>]),*)
            where
                $([<F $letter>]: Fetch<'a, T, [<M $letter>]>,)*
            {
                fn fetch(resources: &'a T) -> Self {
                    ($([<F $letter>]::fetch(resources)),*)
                }

                #[allow(non_snake_case)]
                unsafe fn release(resources: &'a T) {
                    $([<F $letter>]::release(resources);)*
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bits(resource_set: &mut BorrowSet) {
                    $([<F $letter>]::set_resource_bits(resource_set);)*
                }
            }
        }
    }
}

impl_for_tuples!(impl_fetch);

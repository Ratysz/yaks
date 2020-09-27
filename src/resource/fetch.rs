use super::{ContainsMut, ContainsRef};

#[cfg(feature = "parallel")]
use crate::BorrowSet;

/// Specifies how a tuple of types may be borrowed from a tuple of cells.
pub trait Fetch<Resources, Markers>: Sized {
    fn fetch(resources: Resources) -> Self;

    unsafe fn release(resources: Resources);

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet);
}

impl<'a, Resources> Fetch<&'a Resources, ()> for () {
    fn fetch(_: &'a Resources) -> Self {}

    unsafe fn release(_: &'a Resources) {}

    #[cfg(feature = "parallel")]
    fn set_resource_bits(_: &mut BorrowSet) {}
}

impl<'a, Resources, M, R> Fetch<&'a Resources, M> for &'a R
where
    Resources: ContainsRef<R, M>,
    R: 'a,
{
    fn fetch(resources: &'a Resources) -> Self {
        Resources::borrow_ref(resources)
    }

    unsafe fn release(resources: &'a Resources) {
        Resources::release_ref(resources);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet) {
        Resources::set_resource_bit(&mut resource_set.immutable);
    }
}

impl<'a, Resources, M, R> Fetch<&'a Resources, M> for &'a mut R
where
    Resources: ContainsMut<R, M>,
    R: 'a,
{
    fn fetch(resources: &'a Resources) -> Self {
        Resources::borrow_mut(resources)
    }

    unsafe fn release(resources: &'a Resources) {
        Resources::release_mut(resources);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut BorrowSet) {
        Resources::set_resource_bit(&mut resource_set.mutable);
    }
}

macro_rules! impl_fetch {
    ($($letter:ident),*) => {
        paste::item! {
            impl<'a, Resources, $([<M $letter>],)* $([<F $letter>],)*>
                Fetch<&'a Resources, ($([<M $letter>],)*)> for ($([<F $letter>],)*)
            where
                Resources: 'a,
                $([<F $letter>]: Fetch<&'a Resources, [<M $letter>]>,)*
            {
                fn fetch(resources: &'a Resources) -> Self {
                    ($([<F $letter>]::fetch(resources),)*)
                }

                #[allow(non_snake_case)]
                unsafe fn release(resources: &'a Resources) {
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

impl_for_tuples!(impl_fetch, all);

use super::{Contains, DerefTuple, Ref, RefMut};

#[cfg(feature = "parallel")]
use super::ResourceSet;

pub trait Fetch<'a, T, M0>: Sized {
    type Fetched: DerefTuple<'a, Output = Self>;

    fn fetch(resources: &T) -> Self::Fetched;

    fn release(resources: &T, fetched: Self::Fetched);

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut ResourceSet);
}

impl<'a, T, M0, R0> Fetch<'a, T, M0> for &'a R0
where
    T: Contains<R0, M0>,
    R0: 'a,
{
    type Fetched = Ref<R0>;

    fn fetch(resources: &T) -> Self::Fetched {
        T::borrow(resources)
    }

    fn release(resources: &T, fetched: Self::Fetched) {
        T::release(resources, fetched);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut ResourceSet) {
        T::set_resource_bit(&mut resource_set.immutable);
    }
}

impl<'a, T, M0, R0> Fetch<'a, T, M0> for &'a mut R0
where
    T: Contains<R0, M0>,
    R0: 'a,
{
    type Fetched = RefMut<R0>;

    fn fetch(resources: &T) -> Self::Fetched {
        T::borrow_mut(resources)
    }

    fn release(resources: &T, fetched: Self::Fetched) {
        T::release_mut(resources, fetched);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut ResourceSet) {
        T::set_resource_bit(&mut resource_set.mutable);
    }
}

impl<'a, T> Fetch<'a, T, ()> for () {
    type Fetched = ();

    fn fetch(_: &T) -> Self::Fetched {}

    fn release(_: &T, _: Self::Fetched) {}

    #[cfg(feature = "parallel")]
    fn set_resource_bits(_: &mut ResourceSet) {}
}

impl<'a, T, M0, F0> Fetch<'a, T, (M0,)> for (F0,)
where
    F0: Fetch<'a, T, M0>,
{
    type Fetched = (F0::Fetched,);

    fn fetch(resources: &T) -> Self::Fetched {
        (F0::fetch(resources),)
    }

    fn release(resources: &T, fetched: Self::Fetched) {
        F0::release(resources, fetched.0);
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bits(resource_set: &mut ResourceSet) {
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
                type Fetched = ($([<F $letter>]::Fetched,)*);

                fn fetch(resources: &T) -> Self::Fetched {
                    ($([<F $letter>]::fetch(resources)),*)
                }

                #[allow(non_snake_case)]
                fn release(resources: &T, fetched: Self::Fetched) {
                    let ($($letter,)*) = fetched;
                    $([<F $letter>]::release(resources, $letter);)*
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bits(resource_set: &mut ResourceSet) {
                    $([<F $letter>]::set_resource_bits(resource_set);)*
                }
            }
        }
    }
}

impl_for_tuples!(impl_fetch);

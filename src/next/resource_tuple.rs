use std::marker::PhantomData;

use super::{AtomicBorrow, ResourceCell};

pub struct WrappedResources<'a, Wrapped> {
    phantom_data: PhantomData<&'a ()>,
    pub(crate) tuple: Wrapped,
}

pub trait ResourceTuple {
    type Cells;
    type Borrows;
    const LENGTH: usize;

    fn instantiate_borrows() -> Self::Borrows;
}

pub trait ResourceWrap {
    type Types: ResourceTuple<Cells = Self::Cells, Borrows = Self::Borrows>;
    type Cells;
    type Borrows;

    fn wrap(&mut self, borrows: &mut Self::Borrows) -> WrappedResources<Self::Cells>;
}

impl<'a, R0> ResourceWrap for &'a mut R0
where
    R0: Send + Sync,
{
    type Types = (R0,);
    type Cells = (ResourceCell<R0>,);
    type Borrows = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::Borrows) -> WrappedResources<Self::Cells> {
        WrappedResources {
            phantom_data: PhantomData,
            tuple: (ResourceCell::new(self, &mut borrows.0),),
        }
    }
}

impl ResourceTuple for () {
    type Cells = ();
    type Borrows = ();
    const LENGTH: usize = 0;

    fn instantiate_borrows() -> Self::Borrows {}
}

impl ResourceWrap for () {
    type Types = ();
    type Cells = ();
    type Borrows = ();

    fn wrap(&mut self, _: &mut Self::Borrows) -> WrappedResources<Self::Cells> {
        WrappedResources {
            phantom_data: PhantomData,
            tuple: (),
        }
    }
}

impl<R0> ResourceTuple for (R0,)
where
    R0: Send + Sync,
{
    type Cells = (ResourceCell<R0>,);
    type Borrows = (AtomicBorrow,);
    const LENGTH: usize = 1;

    fn instantiate_borrows() -> Self::Borrows {
        (AtomicBorrow::new(),)
    }
}

impl<'a, R0> ResourceWrap for (&'a mut R0,)
where
    R0: Send + Sync,
{
    type Types = (R0,);
    type Cells = (ResourceCell<R0>,);
    type Borrows = (AtomicBorrow,);

    fn wrap(&mut self, borrows: &mut Self::Borrows) -> WrappedResources<Self::Cells> {
        WrappedResources {
            phantom_data: PhantomData,
            tuple: (ResourceCell::new(self.0, &mut borrows.0),),
        }
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
            type Cells = ($(ResourceCell<$letter>,)*);
            type Borrows = ($(swap_to_atomic_borrow!($letter),)*);
            const LENGTH: usize = count!($($letter)*);

            fn instantiate_borrows() -> Self::Borrows {
                ($(swap_to_atomic_borrow!(new $letter),)*)
            }
        }
    }
}

impl_for_tuples!(impl_resource_tuple);

macro_rules! impl_resource_wrap {
    ($($letter:ident),*) => {
        paste::item! {
            impl<'a, $($letter),*> ResourceWrap for ($(&'a mut $letter,)*)
            where
                $($letter: Send + Sync,)*
            {
                type Types = ($($letter,)*);
                type Cells = ($(ResourceCell<$letter>,)*);
                type Borrows = ($(swap_to_atomic_borrow!($letter),)*);

                #[allow(non_snake_case)]
                fn wrap(&mut self, borrows: &mut Self::Borrows) -> WrappedResources<Self::Cells> {
                    let ($([<S $letter>],)*) = self;
                    let ($([<B $letter>],)*) = borrows;
                    WrappedResources {
                        phantom_data: PhantomData,
                        tuple: (
                            $( ResourceCell::new([<S $letter>], [<B $letter>]) ,)*
                        )
                    }
                }
            }
        }
    }
}

impl_for_tuples!(impl_resource_wrap);

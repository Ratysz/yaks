#[cfg(feature = "parallel")]
use fixedbitset::FixedBitSet;

use super::{ResourceMutCell, ResourceRefCell};

pub trait ContainsRef<R, M> {
    fn borrow_ref(&self) -> &R;

    unsafe fn release_ref(&self);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

pub trait ContainsMut<R, M> {
    #[allow(clippy::mut_from_ref)]
    fn borrow_mut(&self) -> &mut R;

    unsafe fn release_mut(&self);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

impl<R> ContainsRef<R, ()> for ResourceRefCell<R>
where
    R: Send + Sync,
{
    fn borrow_ref(&self) -> &R {
        self.cell_borrow_ref()
    }

    unsafe fn release_ref(&self) {
        self.cell_release_ref()
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(_: &mut FixedBitSet) {
        unreachable!()
    }
}

impl<R> ContainsRef<R, ()> for ResourceMutCell<R>
where
    R: Send + Sync,
{
    fn borrow_ref(&self) -> &R {
        self.cell_borrow_ref()
    }

    unsafe fn release_ref(&self) {
        self.cell_release_ref()
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(_: &mut FixedBitSet) {
        unreachable!()
    }
}

impl<R> ContainsMut<R, ()> for ResourceMutCell<R>
where
    R: Send + Sync,
{
    fn borrow_mut(&self) -> &mut R {
        self.cell_borrow_mut()
    }

    unsafe fn release_mut(&self) {
        self.cell_release_mut()
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(_: &mut FixedBitSet) {
        unreachable!()
    }
}

macro_rules! swap_to_unit {
    ($anything:tt) => {
        ()
    };
}

macro_rules! swap_to_markers {
    ($($letter:ident),*) => {
        ($( swap_to_unit!($letter), )*)
    }
}

macro_rules! impl_contains {
    () => {};
    ($($letter:ident),*) => {
        impl_contains!($($letter),* ; $($letter),*);
    };
    ($($all:ident),* ; $letter:ident ) => {
        impl_contains!($($all),* ; $letter ; );
    };
    ($($all:ident),* ; $letter:ident, $($tail:ident),*) => {
        impl_contains!($($all),* ; $letter ; $($tail),*);
        impl_contains!($($all),* ; $($tail),*);
    };
    ($($all:ident),* ; $letter:ident ; $($tail:ident),*) => {
        paste::item! {
            #[allow(unused_variables)]
            impl<[<Resource $letter>], $([<Cell $all>],)*>
                ContainsRef<
                    [<Resource $letter>],
                    ([<Resource $letter>], swap_to_markers!($($tail),*))
                > for ($([<Cell $all>],)*)
            where [<Cell $letter>]: ContainsRef<[<Resource $letter>], ()>
            {
                fn borrow_ref(&self) -> &[<Resource $letter>] {
                    let ($([<cell_$all:lower>],)*) = self;
                    [<cell_$letter:lower>].borrow_ref()
                }

                unsafe fn release_ref(&self) {
                    let ($([<cell_$all:lower>],)*) = self;
                    [<cell_$letter:lower>].release_ref()
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bit(bitset: &mut FixedBitSet) {
                    bitset.insert(count!($($all)*) - (1usize + count!($($tail)*)));
                }
            }

            #[allow(unused_variables)]
            impl<[<Resource $letter>], $([<Cell $all>],)*>
                ContainsMut<
                    [<Resource $letter>],
                    ([<Resource $letter>], swap_to_markers!($($tail),*))
                > for ($([<Cell $all>],)*)
            where [<Cell $letter>]: ContainsMut<[<Resource $letter>], ()>
            {
                fn borrow_mut(&self) -> &mut [<Resource $letter>] {
                    let ($([<cell_$all:lower>],)*) = self;
                    [<cell_$letter:lower>].borrow_mut()
                }

                unsafe fn release_mut(&self) {
                    let ($([<cell_$all:lower>],)*) = self;
                    [<cell_$letter:lower>].release_mut()
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bit(bitset: &mut FixedBitSet) {
                    bitset.insert(count!($($all)*) - (1usize + count!($($tail)*)));
                }
            }
        }
    };
}

impl_for_tuples!(impl_contains);

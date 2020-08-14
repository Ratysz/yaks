#[cfg(feature = "parallel")]
use fixedbitset::FixedBitSet;

use super::{ResourceCell, ResourceCellMut, ResourceCellRef};

pub trait ContainsRef<R0, M0> {
    fn borrow_ref(&self) -> &R0;

    unsafe fn release_ref(&self);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

pub trait ContainsMut<R0, M0> {
    #[allow(clippy::mut_from_ref)]
    fn borrow_mut(&self) -> &mut R0;

    unsafe fn release_mut(&self);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

impl<R0> ContainsRef<R0, ()> for (ResourceCellRef<R0>,)
where
    R0: Send + Sync,
{
    fn borrow_ref(&self) -> &R0 {
        self.0.borrow_ref()
    }

    unsafe fn release_ref(&self) {
        self.0.release_ref();
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet) {
        bitset.insert(0);
    }
}

impl<R0> ContainsRef<R0, ()> for (ResourceCellMut<R0>,)
where
    R0: Send + Sync,
{
    fn borrow_ref(&self) -> &R0 {
        self.0.borrow_ref()
    }

    unsafe fn release_ref(&self) {
        self.0.release_ref();
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet) {
        bitset.insert(0);
    }
}

impl<R0> ContainsMut<R0, ()> for (ResourceCellMut<R0>,)
where
    R0: Send + Sync,
{
    fn borrow_mut(&self) -> &R0 {
        self.0.borrow_mut()
    }

    unsafe fn release_mut(&self) {
        self.0.release_mut();
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet) {
        bitset.insert(0);
    }
}

/// Specifies how a specific type may be borrowed from a tuple of cells.
pub trait Contains<R0, M0> {
    fn borrow(&self) -> &R0;

    #[allow(clippy::mut_from_ref)]
    fn borrow_mut(&self) -> &mut R0;

    unsafe fn release(&self);

    unsafe fn release_mut(&self);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

impl<R0, C0> Contains<R0, ()> for (C0,)
where
    C0: ResourceCell<R0>,
{
    fn borrow(&self) -> &R0 {
        self.0.borrow_ref()
    }

    fn borrow_mut(&self) -> &mut R0 {
        self.0.borrow_mut()
    }

    unsafe fn release(&self) {
        self.0.release_ref();
    }

    unsafe fn release_mut(&self) {
        self.0.release_mut();
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet) {
        bitset.insert(0);
    }
}

/*macro_rules! swap_to_unit {
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
    ($($letter:ident),*) => {
        impl_contains!($($letter),* ; $($letter),*);
    };
    ($($all:ident),* ; $letter:ident, $($tail:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<$($all),*> Contains<$letter, ($letter, swap_to_markers!($($tail),*))>
            for ($(ResourceCell<$all>,)*)
        {
            fn borrow(&self) -> &$letter {
                let ($($all,)*) = self;
                $letter.borrow_ref()
            }

            fn borrow_mut(&self) -> &mut $letter {
                let ($($all,)*) = self;
                $letter.borrow_mut()
            }

            unsafe fn release(&self) {
                let ($($all,)*) = self;
                $letter.release_ref();
            }

            unsafe fn release_mut(&self) {
                let ($($all,)*) = self;
                $letter.release_mut();
            }

            #[cfg(feature = "parallel")]
            fn set_resource_bit(bitset: &mut FixedBitSet) {
                bitset.insert(count!($($all)*) - (1usize + count!($($tail)*)));
            }
        }
        impl_contains!($($all),* ; $($tail),*);
    };
    ($($all:ident),* ; $letter:ident ) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<$($all),*> Contains<$letter, ($letter, )>
            for ($(ResourceCell<$all>,)*)
        {
            fn borrow(&self) -> &$letter {
                let ($($all,)*) = self;
                $letter.borrow_ref()
            }

            fn borrow_mut(&self) -> &mut $letter {
                let ($($all,)*) = self;
                $letter.borrow_mut()
            }

            unsafe fn release(&self) {
                let ($($all,)*) = self;
                $letter.release_ref();
            }

            unsafe fn release_mut(&self) {
                let ($($all,)*) = self;
                $letter.release_mut();
            }

            #[cfg(feature = "parallel")]
            fn set_resource_bit(bitset: &mut FixedBitSet) {
                bitset.insert(count!($($all)*) - 1usize);
            }
        }
    }
}

impl_for_tuples!(impl_contains);*/

#[cfg(feature = "parallel")]
use fixedbitset::FixedBitSet;

use super::ResourceCell;

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

impl<R0> Contains<R0, ()> for (ResourceCell<R0>,) {
    fn borrow(&self) -> &R0 {
        self.0.borrow()
    }

    fn borrow_mut(&self) -> &mut R0 {
        self.0.borrow_mut()
    }

    unsafe fn release(&self) {
        self.0.release();
    }

    unsafe fn release_mut(&self) {
        self.0.release_mut();
    }

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet) {
        bitset.insert(0);
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
                $letter.borrow()
            }

            fn borrow_mut(&self) -> &mut $letter {
                let ($($all,)*) = self;
                $letter.borrow_mut()
            }

            unsafe fn release(&self) {
                let ($($all,)*) = self;
                $letter.release();
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
                $letter.borrow()
            }

            fn borrow_mut(&self) -> &mut $letter {
                let ($($all,)*) = self;
                $letter.borrow_mut()
            }

            unsafe fn release(&self) {
                let ($($all,)*) = self;
                $letter.release();
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

impl_for_tuples!(impl_contains);

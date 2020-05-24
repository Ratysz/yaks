#[cfg(feature = "parallel")]
use fixedbitset::FixedBitSet;

use crate::{Ref, RefMut, ResourceCell, WrappedResources};

pub trait Contains<R0, M0> {
    fn borrow(&self) -> Ref<R0>;

    fn borrow_mut(&self) -> RefMut<R0>;

    fn release(&self, borrowed: Ref<R0>);

    fn release_mut(&self, borrowed: RefMut<R0>);

    #[cfg(feature = "parallel")]
    fn set_resource_bit(bitset: &mut FixedBitSet);
}

impl<R0> Contains<R0, ()> for WrappedResources<'_, (ResourceCell<R0>,)> {
    fn borrow(&self) -> Ref<R0> {
        self.tuple.0.borrow()
    }

    fn borrow_mut(&self) -> RefMut<R0> {
        self.tuple.0.borrow_mut()
    }

    fn release(&self, borrowed: Ref<R0>) {
        self.tuple.0.release(borrowed);
    }

    fn release_mut(&self, borrowed: RefMut<R0>) {
        self.tuple.0.release_mut(borrowed);
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
            for WrappedResources<'_, ($(ResourceCell<$all>,)*)>
        {
            fn borrow(&self) -> Ref<$letter> {
                let ($($all,)*) = &self.tuple;
                $letter.borrow()
            }

            fn borrow_mut(&self) -> RefMut<$letter> {
                let ($($all,)*) = &self.tuple;
                $letter.borrow_mut()
            }

            fn release(&self, borrowed: Ref<$letter>) {
                let ($($all,)*) = &self.tuple;
                $letter.release(borrowed);
            }

            fn release_mut(&self, borrowed: RefMut<$letter>) {
                let ($($all,)*) = &self.tuple;
                $letter.release_mut(borrowed);
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
            for WrappedResources<'_, ($(ResourceCell<$all>,)*)>
        {
            fn borrow(&self) -> Ref<$letter> {
                let ($($all,)*) = &self.tuple;
                $letter.borrow()
            }

            fn borrow_mut(&self) -> RefMut<$letter> {
                let ($($all,)*) = &self.tuple;
                $letter.borrow_mut()
            }

            fn release(&self, borrowed: Ref<$letter>) {
                let ($($all,)*) = &self.tuple;
                $letter.release(borrowed);
            }

            fn release_mut(&self, borrowed: RefMut<$letter>) {
                let ($($all,)*) = &self.tuple;
                $letter.release_mut(borrowed);
            }

            #[cfg(feature = "parallel")]
            fn set_resource_bit(bitset: &mut FixedBitSet) {
                bitset.insert(count!($($all)*) - 1usize);
            }
        }
    }
}

impl_for_tuples!(impl_contains);

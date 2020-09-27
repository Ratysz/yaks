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
    ($($letter:ident),*) => {
        impl_contains!($($letter),* ; $($letter),*);
    };
    ($($all:ident),* ; $letter:ident, $($tail:ident),*) => {
        paste::item! {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            impl<$letter, $([<C $all>],)*>
                ContainsRef<$letter, ($letter, swap_to_markers!($($tail),*))>
                for ($([<C $all>],)*)
            where [<C $letter>]: ContainsRef<$letter, ()>
            {
                fn borrow_ref(&self) -> &$letter {
                    let ($($all,)*) = self;
                    $letter.borrow_ref()
                }

                unsafe fn release_ref(&self) {
                    let ($($all,)*) = self;
                    $letter.release_ref();
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bit(bitset: &mut FixedBitSet) {
                    bitset.insert(count!($($all)*) - (1usize + count!($($tail)*)));
                }
            }

            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            impl<$letter, $([<C $all>],)*>
                ContainsMut<$letter, ($letter, swap_to_markers!($($tail),*))>
                for ($([<C $all>],)*)
            where [<C $letter>]: ContainsMut<$letter, ()>
            {
                fn borrow_mut(&self) -> &mut $letter {
                    let ($($all,)*) = self;
                    $letter.borrow_mut()
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
        }

        impl_contains!($($all),* ; $($tail),*);
    };
    ($($all:ident),* ; $letter:ident ) => {
        paste::item! {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            impl<$letter, $([<C $all>],)*>
                ContainsRef<$letter, ($letter, )>
                for ($([<C $all>],)*)
            where [<C $letter>]: ContainsRef<$letter, ()>
            {
                fn borrow_ref(&self) -> &$letter {
                    let ($($all,)*) = self;
                    $letter.borrow_ref()
                }

                unsafe fn release_ref(&self) {
                    let ($($all,)*) = self;
                    $letter.release_ref();
                }

                #[cfg(feature = "parallel")]
                fn set_resource_bit(bitset: &mut FixedBitSet) {
                    bitset.insert(count!($($all)*) - 1usize);
                }
            }

            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            impl<$letter, $([<C $all>],)*>
                ContainsMut<$letter, ($letter, )>
                for ($([<C $all>],)*)
            where [<C $letter>]: ContainsMut<$letter, ()>
            {
                fn borrow_mut(&self) -> &mut $letter {
                    let ($($all,)*) = self;
                    $letter.borrow_mut()
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
}

impl_for_tuples!(impl_contains, all);

use std::{ptr::NonNull, thread::panicking};

use crate::AtomicBorrow;

pub struct ResourceCell<R0> {
    cell: NonNull<R0>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R0> ResourceCell<R0> {
    pub fn new(resource: &mut R0, borrow: &mut AtomicBorrow) -> Self
    where
        R0: Send + Sync,
    {
        Self {
            cell: NonNull::new(resource).expect("pointers to resources should never be null"),
            borrow: NonNull::new(borrow).expect("pointers to AtomicBorrows should never be null"),
        }
    }

    pub fn borrow(&self) -> &R0 {
        assert!(
            unsafe { self.borrow.as_ref().borrow() },
            "cannot borrow {} immutably: already borrowed mutably",
            std::any::type_name::<R0>()
        );
        unsafe { self.cell.as_ref() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn borrow_mut(&self) -> &mut R0 {
        assert!(
            unsafe { self.borrow.as_ref().borrow_mut() },
            "cannot borrow {} mutably: already borrowed",
            std::any::type_name::<R0>()
        );
        unsafe { &mut *self.cell.clone().as_ptr() }
    }

    pub unsafe fn release(&self) {
        self.borrow.as_ref().release();
    }

    pub unsafe fn release_mut(&self) {
        self.borrow.as_ref().release_mut();
    }
}

impl<R0> Drop for ResourceCell<R0> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !panicking() {
            assert!(
                unsafe { self.borrow.as_ref().is_free() },
                "borrows of {} were not released properly",
                std::any::type_name::<R0>()
            )
        }
    }
}

unsafe impl<R0> Send for ResourceCell<R0> where R0: Send {}

unsafe impl<R0> Sync for ResourceCell<R0> where R0: Sync {}

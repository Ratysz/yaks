use std::{ptr::NonNull, thread::panicking};

use super::AtomicBorrow;

pub struct ResourceRefCell<R> {
    pointer: NonNull<R>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R> ResourceRefCell<R>
where
    R: Send + Sync,
{
    pub fn new(resource: &R, borrow: &mut AtomicBorrow) -> Self {
        Self {
            pointer: resource.into(),
            borrow: borrow.into(),
        }
    }

    pub fn cell_borrow_ref(&self) -> &R {
        unsafe {
            if !self.borrow.as_ref().borrow() {
                unreachable!(
                    "could not immutably borrow {} from an immutable resource cell",
                    std::any::type_name::<R>()
                )
            }
            self.pointer.as_ref()
        }
    }

    pub unsafe fn cell_release_ref(&self) {
        self.borrow.as_ref().release();
    }
}

unsafe impl<R> Send for ResourceRefCell<R> where R: Send {}

unsafe impl<R> Sync for ResourceRefCell<R> where R: Sync {}

impl<R> Drop for ResourceRefCell<R> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !panicking() {
            assert!(
                unsafe { self.borrow.as_ref().is_free() },
                "borrows of {} in an immutable resource cell were not released properly",
                std::any::type_name::<R>()
            )
        }
    }
}

pub struct ResourceMutCell<R> {
    pointer: NonNull<R>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R> ResourceMutCell<R>
where
    R: Send + Sync,
{
    pub fn new(resource: &mut R, borrow: &mut AtomicBorrow) -> Self {
        Self {
            pointer: resource.into(),
            borrow: borrow.into(),
        }
    }

    pub fn cell_borrow_ref(&self) -> &R {
        unsafe {
            assert!(
                self.borrow.as_ref().borrow(),
                "cannot borrow {} immutably: already borrowed mutably",
                std::any::type_name::<R>()
            );
            self.pointer.as_ref()
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn cell_borrow_mut(&self) -> &mut R {
        unsafe {
            assert!(
                self.borrow.as_ref().borrow_mut(),
                "cannot borrow {} mutably: already borrowed",
                std::any::type_name::<R>()
            );
            &mut *self.pointer.clone().as_ptr()
        }
    }

    pub unsafe fn cell_release_ref(&self) {
        self.borrow.as_ref().release();
    }

    pub unsafe fn cell_release_mut(&self) {
        self.borrow.as_ref().release_mut();
    }
}

unsafe impl<R> Send for ResourceMutCell<R> where R: Send {}

unsafe impl<R> Sync for ResourceMutCell<R> where R: Sync {}

impl<R> Drop for ResourceMutCell<R> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !panicking() {
            assert!(
                unsafe { self.borrow.as_ref().is_free() },
                "borrows of {} in a mutable resource cell were not released properly",
                std::any::type_name::<R>()
            )
        }
    }
}

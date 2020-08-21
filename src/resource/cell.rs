use std::{ptr::NonNull, thread::panicking};

use super::AtomicBorrow;

pub struct ResourceRefCell<R0> {
    pointer: NonNull<R0>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R0> ResourceRefCell<R0>
where
    R0: Send + Sync,
{
    pub fn new(resource: &R0, borrow: &mut AtomicBorrow) -> Self {
        Self {
            pointer: resource.into(),
            borrow: borrow.into(),
        }
    }
}

unsafe impl<R0> Send for ResourceRefCell<R0> where R0: Send {}

unsafe impl<R0> Sync for ResourceRefCell<R0> where R0: Sync {}

impl<R0> Drop for ResourceRefCell<R0> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !panicking() {
            assert!(
                unsafe { self.borrow.as_ref().is_free() },
                "borrows of {} in an immutable resource cell were not released properly",
                std::any::type_name::<R0>()
            )
        }
    }
}

pub struct ResourceMutCell<R0> {
    pointer: NonNull<R0>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R0> ResourceMutCell<R0>
where
    R0: Send + Sync,
{
    pub fn new(resource: &mut R0, borrow: &mut AtomicBorrow) -> Self {
        Self {
            pointer: resource.into(),
            borrow: borrow.into(),
        }
    }
}

unsafe impl<R0> Send for ResourceMutCell<R0> where R0: Send {}

unsafe impl<R0> Sync for ResourceMutCell<R0> where R0: Sync {}

impl<R0> Drop for ResourceMutCell<R0> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !panicking() {
            assert!(
                unsafe { self.borrow.as_ref().is_free() },
                "borrows of {} in a mutable resource cell were not released properly",
                std::any::type_name::<R0>()
            )
        }
    }
}

pub trait ResourceCell<R0> {
    fn borrow_ref(&self) -> &R0;

    #[allow(clippy::mut_from_ref)]
    fn borrow_mut(&self) -> &mut R0;

    unsafe fn release_ref(&self);

    unsafe fn release_mut(&self);
}

impl<R0> ResourceCell<R0> for ResourceRefCell<R0>
where
    R0: Send + Sync,
{
    fn borrow_ref(&self) -> &R0 {
        unsafe {
            if !self.borrow.as_ref().borrow() {
                unreachable!(
                    "could not immutably borrow {} from an immutable resource cell",
                    std::any::type_name::<R0>()
                )
            }
            self.pointer.as_ref()
        }
    }

    fn borrow_mut(&self) -> &mut R0 {
        panic!(
            "attempted to mutably borrow {} which is specified as immutable in executor signature",
            std::any::type_name::<R0>()
        );
    }

    unsafe fn release_ref(&self) {
        self.borrow.as_ref().release();
    }

    unsafe fn release_mut(&self) {
        unreachable!(
            "attempted to release a mutable borrow of {} in an immutable resource cell",
            std::any::type_name::<R0>()
        );
    }
}

impl<R0> ResourceCell<R0> for ResourceMutCell<R0>
where
    R0: Send + Sync,
{
    fn borrow_ref(&self) -> &R0 {
        unsafe {
            assert!(
                self.borrow.as_ref().borrow(),
                "cannot borrow {} immutably: already borrowed mutably",
                std::any::type_name::<R0>()
            );
            self.pointer.as_ref()
        }
    }

    fn borrow_mut(&self) -> &mut R0 {
        unsafe {
            assert!(
                self.borrow.as_ref().borrow_mut(),
                "cannot borrow {} mutably: already borrowed",
                std::any::type_name::<R0>()
            );
            &mut *self.pointer.clone().as_ptr()
        }
    }

    unsafe fn release_ref(&self) {
        self.borrow.as_ref().release();
    }

    unsafe fn release_mut(&self) {
        self.borrow.as_ref().release_mut();
    }
}

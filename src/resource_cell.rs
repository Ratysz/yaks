use std::ptr::NonNull;

use super::AtomicBorrow;

pub struct ResourceCell<R0> {
    cell: NonNull<R0>,
    borrow: NonNull<AtomicBorrow>,
}

impl<R0> ResourceCell<R0> {
    pub(crate) fn new(resource: &mut R0, borrow: &mut AtomicBorrow) -> Self
    where
        R0: Send + Sync,
    {
        Self {
            cell: NonNull::new(resource).expect("pointers to resources should never be null"),
            borrow: NonNull::new(borrow).expect("pointers to AtomicBorrows should never be null"),
        }
    }

    pub(crate) fn borrow(&self) -> Ref<R0> {
        assert!(
            unsafe { self.borrow.as_ref().borrow() },
            "cannot borrow {} immutably: already borrowed mutably",
            std::any::type_name::<R0>()
        );
        Ref(self.cell)
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<R0> {
        assert!(
            unsafe { self.borrow.as_ref().borrow_mut() },
            "cannot borrow {} mutably: already borrowed",
            std::any::type_name::<R0>()
        );
        RefMut(self.cell)
    }

    pub(crate) fn release(&self, _: Ref<R0>) {
        unsafe { self.borrow.as_ref().release() };
    }

    pub(crate) fn release_mut(&self, _: RefMut<R0>) {
        unsafe { self.borrow.as_ref().release_mut() };
    }
}

impl<R0> Drop for ResourceCell<R0> {
    fn drop(&mut self) {
        #[cfg(not(feature = "test"))]
        debug_assert!(
            unsafe { self.borrow.as_ref().is_free() },
            "borrows of {} were not released properly",
            std::any::type_name::<R0>()
        )
    }
}

unsafe impl<R0> Send for ResourceCell<R0> where R0: Send {}

unsafe impl<R0> Sync for ResourceCell<R0> where R0: Sync {}

pub struct Ref<R0>(NonNull<R0>);

impl<R0> Ref<R0> {
    pub(crate) fn deref(&self) -> &R0 {
        unsafe { self.0.as_ref() }
    }
}

pub struct RefMut<R0>(NonNull<R0>);

impl<R0> RefMut<R0> {
    pub(crate) fn deref(&mut self) -> &mut R0 {
        unsafe { self.0.as_mut() }
    }
}

#[cfg(feature = "parallel")]
use std::sync::{Arc, Mutex};
use std::{hash::Hash, ops::DerefMut};

#[cfg(feature = "parallel")]
use crate::borrows::{ArchetypeSet, CondensedBorrows, SystemBorrows};
use crate::System;

#[cfg(not(feature = "parallel"))]
pub(crate) struct SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    system: System,
    pub dependencies: Vec<H>,
    pub active: bool,
}

#[cfg(not(feature = "parallel"))]
impl<H> SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    pub fn new(system: System, dependencies: Vec<H>) -> Self {
        Self {
            system,
            dependencies,
            active: true,
        }
    }

    pub fn unwrap(self) -> System {
        self.system
    }

    pub fn system_mut(&mut self) -> impl DerefMut<Target = System> + '_ {
        &mut self.system
    }
}

#[cfg(feature = "parallel")]
pub(crate) struct SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    pub system: Arc<Mutex<System>>,
    pub dependencies: Vec<H>,
    pub active: bool,
    pub borrows: SystemBorrows,
    pub condensed: CondensedBorrows,
    pub archetypes: ArchetypeSet,
}

#[cfg(feature = "parallel")]
impl<H> SystemContainer<H>
where
    H: Hash + Eq + PartialEq,
{
    pub fn new(system: System, dependencies: Vec<H>) -> Self {
        let mut borrows = SystemBorrows::default();
        system.inner().write_borrows(&mut borrows);
        let archetypes = ArchetypeSet::default();
        Self {
            system: Arc::new(Mutex::new(system)),
            dependencies,
            active: true,
            borrows,
            condensed: CondensedBorrows::with_capacity(0, 0),
            archetypes,
        }
    }

    pub fn unwrap(self) -> System {
        match Arc::try_unwrap(self.system) {
            Ok(mutex) => mutex
                .into_inner()
                .expect("mutexes should never be poisoned"),
            Err(_) => {
                unreachable!("unwrapping a system container should only happen in a sync scope")
            }
        }
    }

    pub fn system_mut(&mut self) -> impl DerefMut<Target = System> + '_ {
        self.system
            .lock()
            .expect("mutexes should never be poisoned")
    }
}

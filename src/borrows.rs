use fixedbitset::FixedBitSet;
use fxhash::FxHasher64;
use hecs::Access;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault};

#[cfg(feature = "parallel")]
use crate::System;

pub type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;

#[derive(Default)]
pub struct SystemBorrows {
    pub resources_immutable: TypeSet,
    pub resources_mutable: TypeSet,
    pub components_immutable: TypeSet,
    pub components_mutable: TypeSet,
}

#[cfg(feature = "parallel")]
#[derive(Default)]
pub(crate) struct CondensedBorrows {
    pub resources_immutable: FixedBitSet,
    pub resources_mutable: FixedBitSet,
    pub components_immutable: FixedBitSet,
    pub components_mutable: FixedBitSet,
}

#[cfg(feature = "parallel")]
impl CondensedBorrows {
    pub fn fill(
        &mut self,
        borrows: &SystemBorrows,
        all_resources: &TypeSet,
        all_components: &TypeSet,
    ) {
        all_resources.iter().enumerate().for_each(|(i, resource)| {
            if borrows.resources_mutable.contains(resource) {
                self.resources_mutable.insert(i);
            }
            if borrows.resources_immutable.contains(resource) {
                self.resources_immutable.insert(i);
            }
        });
        all_components
            .iter()
            .enumerate()
            .for_each(|(i, component)| {
                if borrows.components_mutable.contains(component) {
                    self.components_mutable.insert(i);
                }
                if borrows.components_immutable.contains(component) {
                    self.components_immutable.insert(i);
                }
            });
    }

    pub fn are_resources_compatible(&self, other: &CondensedBorrows) -> bool {
        self.resources_mutable.is_disjoint(&other.resources_mutable)
            && self
                .resources_mutable
                .is_disjoint(&other.resources_immutable)
            && self
                .resources_immutable
                .is_disjoint(&other.resources_mutable)
    }

    pub fn are_components_compatible(&self, other: &CondensedBorrows) -> bool {
        self.components_mutable
            .is_disjoint(&other.components_mutable)
            && self
                .components_mutable
                .is_disjoint(&other.components_immutable)
            && self
                .components_immutable
                .is_disjoint(&other.components_mutable)
    }

    pub fn clear(&mut self) {
        self.resources_immutable.clear();
        self.resources_mutable.clear();
        self.components_immutable.clear();
        self.components_mutable.clear();
    }

    pub fn grow(&mut self, resources: usize, components: usize) {
        self.resources_immutable.grow(resources);
        self.resources_mutable.grow(resources);
        self.components_immutable.grow(components);
        self.components_mutable.grow(components);
    }
}

#[derive(Default)]
pub struct ArchetypeAccess {
    pub immutable: FixedBitSet,
    pub mutable: FixedBitSet,
}

#[cfg(feature = "parallel")]
impl ArchetypeAccess {
    pub(crate) fn is_compatible(&self, other: &ArchetypeAccess) -> bool {
        self.mutable.is_disjoint(&other.mutable)
            && self.mutable.is_disjoint(&other.immutable)
            && self.immutable.is_disjoint(&other.mutable)
    }

    pub(crate) fn clear(&mut self) {
        self.immutable.clear();
        self.mutable.clear();
    }

    pub(crate) fn grow(&mut self, bits: usize) {
        self.immutable.grow(bits);
        self.mutable.grow(bits);
    }
}

impl Extend<(usize, Access)> for ArchetypeAccess {
    fn extend<T: IntoIterator<Item = (usize, Access)>>(&mut self, iterator: T) {
        iterator
            .into_iter()
            .for_each(|(archetype, access)| match access {
                Access::Read => self.immutable.set(archetype, true),
                Access::Write => self.mutable.set(archetype, true),
                Access::Iterate => (),
            })
    }
}

#[cfg(feature = "parallel")]
pub(crate) struct BorrowsContainer {
    pub borrows: SystemBorrows,
    pub condensed: CondensedBorrows,
    pub archetypes: ArchetypeAccess,
}

#[cfg(feature = "parallel")]
impl BorrowsContainer {
    pub fn new(system: &System) -> Self {
        let mut borrows = SystemBorrows::default();
        system.inner().write_borrows(&mut borrows);
        Self {
            borrows,
            condensed: Default::default(),
            archetypes: Default::default(),
        }
    }
}

use fixedbitset::FixedBitSet;
use fxhash::FxHasher64;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault};

use crate::System;

pub type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;

#[derive(Default)]
pub struct ArchetypeSet {
    bitset: FixedBitSet,
}

impl ArchetypeSet {
    pub fn insert(&mut self, archetype: u32) {
        assert!(archetype < std::usize::MAX as u32);
        let archetype = archetype as usize;
        self.bitset.grow(archetype + 1);
        self.bitset.insert(archetype);
    }

    pub fn clear(&mut self) {
        self.bitset.clear();
    }

    pub(crate) fn as_bitset(&self) -> &FixedBitSet {
        &self.bitset
    }
}

impl Extend<u32> for ArchetypeSet {
    fn extend<T: IntoIterator<Item = u32>>(&mut self, iterator: T) {
        iterator.into_iter().for_each(|value| self.insert(value));
    }
}

#[derive(Default)]
pub struct SystemBorrows {
    pub resources_immutable: TypeSet,
    pub resources_mutable: TypeSet,
    pub components_immutable: TypeSet,
    pub components_mutable: TypeSet,
}

impl SystemBorrows {
    pub(crate) fn condense(
        &self,
        all_resources: &TypeSet,
        all_components: &TypeSet,
    ) -> CondensedBorrows {
        let mut condensed =
            CondensedBorrows::with_capacity(all_resources.len(), all_components.len());
        all_resources.iter().enumerate().for_each(|(i, resource)| {
            if self.resources_mutable.contains(resource) {
                condensed.resources_mutable.insert(i);
            }
            if self.resources_immutable.contains(resource) {
                condensed.resources_immutable.insert(i);
            }
        });
        all_components
            .iter()
            .enumerate()
            .for_each(|(i, component)| {
                if self.components_mutable.contains(component) {
                    condensed.components_mutable.insert(i);
                }
                if self.components_immutable.contains(component) {
                    condensed.components_immutable.insert(i);
                }
            });
        condensed
    }
}

#[derive(Clone)]
pub(crate) struct CondensedBorrows {
    pub resources_immutable: FixedBitSet,
    pub resources_mutable: FixedBitSet,
    pub components_immutable: FixedBitSet,
    pub components_mutable: FixedBitSet,
}

impl CondensedBorrows {
    pub fn with_capacity(resources: usize, components: usize) -> Self {
        Self {
            resources_immutable: FixedBitSet::with_capacity(resources),
            resources_mutable: FixedBitSet::with_capacity(resources),
            components_immutable: FixedBitSet::with_capacity(components),
            components_mutable: FixedBitSet::with_capacity(components),
        }
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
}

pub(crate) struct BorrowsContainer {
    pub borrows: SystemBorrows,
    pub condensed: CondensedBorrows,
    pub archetypes: ArchetypeSet,
}

impl BorrowsContainer {
    pub fn new(system: &System) -> Self {
        let mut borrows = SystemBorrows::default();
        system.inner().write_borrows(&mut borrows);
        Self {
            borrows,
            condensed: CondensedBorrows::with_capacity(0, 0),
            archetypes: ArchetypeSet::default(),
        }
    }
}

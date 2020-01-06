use fxhash::FxHasher64;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault};

use crate::System;

pub type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;
pub type ArchetypeSet = HashSet<u32, BuildHasherDefault<FxHasher64>>;

#[derive(Default, Debug, Clone)]
pub struct SystemBorrows {
    pub resources_immutable: TypeSet,
    pub resources_mutable: TypeSet,
    pub components_immutable: TypeSet,
    pub components_mutable: TypeSet,
}

impl SystemBorrows {
    pub fn are_resource_borrows_compatible(&self, immutable: &TypeSet, mutable: &TypeSet) -> bool {
        self.resources_mutable.is_disjoint(mutable)
            && self.resources_immutable.is_disjoint(mutable)
            && self.resources_mutable.is_disjoint(immutable)
    }

    pub fn are_component_borrows_compatible(&self, immutable: &TypeSet, mutable: &TypeSet) -> bool {
        self.components_mutable.is_disjoint(mutable)
            && self.components_immutable.is_disjoint(mutable)
            && self.components_mutable.is_disjoint(immutable)
    }
}

pub struct SystemWithBorrows {
    pub system: Box<dyn System>,
    pub borrows: SystemBorrows,
}

impl SystemWithBorrows {
    pub fn new(system: Box<dyn System>) -> Self {
        let mut borrows = Default::default();
        system.write_borrows(&mut borrows);
        Self { system, borrows }
    }
}

#[test]
fn test() {
    let mut resources_immutable = TypeSet::default();
    let mut resources_mutable = TypeSet::default();
    let mut borrows = SystemBorrows::default();
}

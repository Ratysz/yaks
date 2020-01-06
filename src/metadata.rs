use fxhash::FxHasher64;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault};

use crate::System;

pub type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;
pub type ArchetypeSet = HashSet<u32, BuildHasherDefault<FxHasher64>>;

#[derive(Default, Debug, Clone)]
pub struct SystemMetadata {
    pub resources_immutable: TypeSet,
    pub resources_mutable: TypeSet,
    pub components_immutable: TypeSet,
    pub components_mutable: TypeSet,
}

impl SystemMetadata {
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

/*impl SystemMetadata {
    pub fn resources_immutable(&self) -> &TypeSet {
        &self.resources_immutable
    }

    pub fn resources_mutable(&self) -> &TypeSet {
        &self.resources_mutable
    }

    pub fn components_immutable(&self) -> &TypeSet {
        &self.components_immutable
    }

    pub fn components_mutable(&self) -> &TypeSet {
        &self.components_mutable
    }
}*/

pub struct SystemWithMetadata {
    pub system: Box<dyn System>,
    pub metadata: SystemMetadata,
}

impl SystemWithMetadata {
    pub fn new(system: Box<dyn System>) -> Self {
        let mut metadata = Default::default();
        system.write_metadata(&mut metadata);
        Self { system, metadata }
    }
}

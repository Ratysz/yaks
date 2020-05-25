use fixedbitset::FixedBitSet;
use hecs::{Access, Query, World};
use std::{any::TypeId, collections::HashSet};

pub type TypeSet = HashSet<TypeId>;

pub struct ComponentTypeSet {
    pub immutable: TypeSet,
    pub mutable: TypeSet,
}

impl ComponentTypeSet {
    pub fn with_capacity(types: usize) -> Self {
        Self {
            immutable: TypeSet::with_capacity(types),
            mutable: TypeSet::with_capacity(types),
        }
    }

    pub fn condense(self, all_component_types: &[TypeId]) -> ComponentSet {
        let mut component_set = ComponentSet::with_capacity(all_component_types.len());
        all_component_types
            .iter()
            .enumerate()
            .for_each(|(i, component)| {
                if self.immutable.contains(component) {
                    component_set.immutable.insert(i);
                }
                if self.mutable.contains(component) {
                    component_set.mutable.insert(i);
                }
            });
        component_set
    }
}

pub struct ComponentSet {
    pub immutable: FixedBitSet,
    pub mutable: FixedBitSet,
}

impl ComponentSet {
    pub fn with_capacity(bits: usize) -> Self {
        Self {
            immutable: FixedBitSet::with_capacity(bits),
            mutable: FixedBitSet::with_capacity(bits),
        }
    }

    pub fn is_compatible(&self, other: &ComponentSet) -> bool {
        self.mutable.is_disjoint(&other.mutable)
            && self.mutable.is_disjoint(&other.immutable)
            && self.immutable.is_disjoint(&other.mutable)
    }
}

pub struct ResourceSet {
    pub immutable: FixedBitSet,
    pub mutable: FixedBitSet,
}

impl ResourceSet {
    pub fn with_capacity(bits: usize) -> Self {
        Self {
            immutable: FixedBitSet::with_capacity(bits),
            mutable: FixedBitSet::with_capacity(bits),
        }
    }

    pub fn is_compatible(&self, other: &ResourceSet) -> bool {
        self.mutable.is_disjoint(&other.mutable)
            && self.mutable.is_disjoint(&other.immutable)
            && self.immutable.is_disjoint(&other.mutable)
    }
}

#[derive(Default)]
pub struct ArchetypeSet {
    pub immutable: FixedBitSet,
    pub mutable: FixedBitSet,
}

impl ArchetypeSet {
    pub fn is_compatible(&self, other: &ArchetypeSet) -> bool {
        self.mutable.is_disjoint(&other.mutable)
            && self.mutable.is_disjoint(&other.immutable)
            && self.immutable.is_disjoint(&other.mutable)
    }

    pub fn set_bits_for_query<Q>(&mut self, world: &World)
    where
        Q: Query,
    {
        self.immutable.clear();
        self.mutable.clear();
        let iterator = world.archetypes();
        let bits = iterator.len();
        self.immutable.grow(bits);
        self.mutable.grow(bits);
        iterator
            .enumerate()
            .filter_map(|(index, archetype)| archetype.access::<Q>().map(|access| (index, access)))
            .for_each(|(archetype, access)| match access {
                Access::Read => self.immutable.set(archetype, true),
                Access::Write => self.mutable.set(archetype, true),
                Access::Iterate => (),
            });
    }
}

use fixedbitset::FixedBitSet;
use hecs::{Access, Query, World};
use std::{any::TypeId, collections::HashSet};

pub type TypeSet = HashSet<TypeId>;

pub struct BorrowTypeSet {
    pub immutable: TypeSet,
    pub mutable: TypeSet,
}

impl BorrowTypeSet {
    // Clippy, this is an internal type that is instantiated in one place, chill.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            immutable: TypeSet::new(),
            mutable: TypeSet::new(),
        }
    }

    pub fn condense(self, all_types: &[TypeId]) -> BorrowSet {
        let mut set = BorrowSet::with_capacity(all_types.len());
        all_types.iter().enumerate().for_each(|(index, element)| {
            if self.immutable.contains(element) {
                set.immutable.insert(index);
            }
            if self.mutable.contains(element) {
                set.mutable.insert(index);
            }
        });
        set
    }
}

pub struct BorrowSet {
    pub immutable: FixedBitSet,
    pub mutable: FixedBitSet,
}

impl BorrowSet {
    pub fn with_capacity(bits: usize) -> Self {
        Self {
            immutable: FixedBitSet::with_capacity(bits),
            mutable: FixedBitSet::with_capacity(bits),
        }
    }

    pub fn is_compatible(&self, other: &BorrowSet) -> bool {
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

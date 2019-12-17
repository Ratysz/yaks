use crate::{system::DynamicSystem, ArchetypeSet, TypeSet};

#[derive(Default)]
struct Stage {
    archetypes: ArchetypeSet,
    components: TypeSet,
    components_mut: TypeSet,
    resources: TypeSet,
    resources_mut: TypeSet,
    systems: Vec<Box<dyn DynamicSystem>>,
}

#[derive(Default)]
pub struct Executor {
    stages: Vec<Stage>,
}

impl Executor {
    pub fn new() -> Self {
        Default::default()
    }
}

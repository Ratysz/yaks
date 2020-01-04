/*use std::{hash::BuildHasherDefault, collections::HashSet, any::TypeId};
use fxhash::FxHasher64;*/

use crate::{ArchetypeSet, System, SystemMetadata, TypeSet, World};

/*type Systems = HashMap<SystemId, Box<dyn System>, BuildHasherDefault<FxHasher64>>;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SystemId(usize);*/

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct ExecutorId(pub usize);

#[derive(Default)]
pub struct Executor {
    stages: Vec<Stage1>,
    id: Option<ExecutorId>,
}

impl Executor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        let metadata = system.metadata();
        for stage in &mut self.stages {
            if stage.is_compatible(&metadata) {
                stage.resources.extend(&metadata.resources);
                stage.resources_mut.extend(&metadata.resources_mut);
                stage.unassigned_tail.push(system);
                break;
            }
        }
    }

    pub fn run(&mut self, world: &mut World) {
        let id = match self.id {
            Some(id) => id,
            None => {
                let id = world.new_executor_id();
                self.id = Some(id);
                id
            }
        };
        if world.executor_needs_rebuilding(id) {
            for stage in &mut self.stages {
                stage.rebuild(world);
                stage.run(world);
            }
        } else {
            for stage in &mut self.stages {
                if !stage.unassigned_tail.is_empty() {
                    stage.rebuild(world);
                }
                stage.run(world);
            }
        }
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &mut World) {
        unimplemented!()
    }
}

#[derive(Default)]
struct Stage1 {
    resources: TypeSet,
    resources_mut: TypeSet,
    stages: Vec<Stage2>,

    unassigned: Vec<Box<dyn System>>,
    unassigned_tail: Vec<Box<dyn System>>,
    archetypes: ArchetypeSet,
}

impl Stage1 {
    fn run(&mut self, world: &World) {
        for stage in &mut self.stages {
            stage.run(world);
        }
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }

    fn is_compatible(&self, metadata: &SystemMetadata) -> bool {
        self.resources_mut.is_disjoint(&metadata.resources_mut)
            && self.resources.is_disjoint(&metadata.resources_mut)
            && self.resources_mut.is_disjoint(&metadata.resources)
    }

    fn rebuild(&mut self, world: &World) {
        for stage in &mut self.stages {
            self.unassigned.extend(stage.systems.drain(..));
            stage.archetypes.clear();
            stage.components.clear();
            stage.components_mut.clear();
        }
        self.unassigned.extend(self.unassigned_tail.drain(..));

        for system in self.unassigned.drain(..) {
            let metadata = system.metadata();
            system.write_touched_archetypes(world, &mut self.archetypes);
            let archetypes = &self.archetypes;
            let stage = match self
                .stages
                .iter_mut()
                .find(|stage| stage.is_compatible(&metadata, archetypes))
            {
                Some(stage) => stage,
                None => {
                    self.stages.push(Stage2::default());
                    self.stages.last_mut().unwrap()
                }
            };
            stage.archetypes.extend(self.archetypes.drain());
            stage.components.extend(&metadata.components);
            stage.components_mut.extend(&metadata.components_mut);
            stage.systems.push(system);
        }
    }
}

#[derive(Default)]
struct Stage2 {
    archetypes: ArchetypeSet,
    components: TypeSet,
    components_mut: TypeSet,
    systems: Vec<Box<dyn System>>,
}

impl Stage2 {
    fn run(&mut self, world: &World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    #[allow(dead_code, unused_variables)]
    fn run_parallel(&mut self, world: &World) {
        unimplemented!()
    }

    fn is_compatible(&self, metadata: &SystemMetadata, archetypes: &ArchetypeSet) -> bool {
        self.archetypes.is_disjoint(archetypes)
            || (self.components_mut.is_disjoint(&metadata.components_mut)
                && self.components.is_disjoint(&metadata.components_mut)
                && self.components_mut.is_disjoint(&metadata.components))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Executor, SystemBuilder};

    struct Resource1;

    struct Resource2;

    struct Resource3;

    struct Component1;

    struct Component2;

    struct Component3;

    struct Component4;

    #[test]
    fn basic() {
        let mut executor = Executor::new();
        executor.add_system(SystemBuilder::<(), ()>::build(|_, _, _| ()));
    }
}

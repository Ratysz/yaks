use hecs::World;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

use crate::{ResourceTuple, ResourceWrap, SystemClosure, SystemContext, SystemId};

/// Parallel executor variant, used when all systems are proven to be statically disjoint,
/// and have no dependencies.
pub struct Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub borrows: Resources::Borrows,
    pub systems: HashMap<SystemId, Arc<Mutex<SystemClosure<'closures, Resources::Cells>>>>,
}

impl<'closures, Resources> Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        let wrapped = resources.wrap(&mut self.borrows);
        self.systems.par_iter().for_each(|(id, system)| {
            let system = &mut *system
                .try_lock() // TODO should this be .lock() instead?
                .expect("systems should only be ran once per execution");
            system(
                SystemContext {
                    system_id: Some(*id),
                    world,
                },
                &wrapped,
            );
        });
    }
}

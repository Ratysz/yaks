use l8r::L8r;
use std::{
    ops::RangeBounds,
    sync::{Arc, Mutex},
};

use crate::World;

pub struct ModificationQueuePool {
    pool: Arc<Mutex<InnerPool>>,
}

struct InnerPool {
    queues: Vec<Option<L8r<World>>>,
}

impl Default for ModificationQueuePool {
    fn default() -> Self {
        ModificationQueuePool::with_capacity(32)
    }
}

impl ModificationQueuePool {
    pub fn with_capacity(capacity: usize) -> Self {
        let queues = std::iter::repeat_with(Default::default)
            .take(capacity)
            .collect();
        let inner = InnerPool { queues };
        Self {
            pool: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn get(&self) -> ModificationQueue {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        let (index, later) = pool
            .queues
            .iter_mut()
            .enumerate()
            .find(|(_, option)| option.is_some())
            .map(|(index, option)| (index, option.take()))
            .unwrap_or_else(|| {
                pool.queues.push(None);
                (pool.queues.len() - 1, Some(Default::default()))
            });
        ModificationQueue {
            later,
            index,
            pool: self.pool.clone(),
        }
    }
}

#[must_use = "dropping a modification queue discards the modifications within"]
pub struct ModificationQueue {
    later: Option<L8r<World>>,
    index: usize,
    pool: Arc<Mutex<InnerPool>>,
}

impl Drop for ModificationQueue {
    fn drop(&mut self) {
        self.get_inner_mut().drain(..);
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool.queues[self.index] = self.later.take();
    }
}

impl ModificationQueue {
    fn get_inner_mut(&mut self) -> &mut L8r<World> {
        self.later
            .as_mut()
            .expect("modification queue should contain a L8r at this point")
    }

    pub fn apply_range<R>(&mut self, world: &mut World, range: R)
    where
        R: RangeBounds<usize>,
    {
        L8r::now(self.get_inner_mut().drain(range), world);
    }

    pub fn apply_all(mut self, world: &mut World) {
        self.apply_range(world, ..);
    }

    pub fn merge(&mut self, mut other: ModificationQueue) {
        self.get_inner_mut().extend(other.get_inner_mut().drain(..));
    }
}

pub trait ModificationQueueBundle: Send + Sync {}

impl ModificationQueueBundle for () {}

impl ModificationQueueBundle for ModificationQueue {}

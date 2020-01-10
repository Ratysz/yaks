use std::{
    iter::{Extend, IntoIterator},
    ops::RangeBounds,
    sync::{Arc, Mutex},
    vec::Drain,
};

use crate::{ComponentBundle, DynamicComponentBundle, Entity, World, WorldProxy};

type Closure = Box<dyn FnOnce(&mut World) + Send + Sync + 'static>;

type InnerQueue = Vec<Closure>;

pub struct ModificationQueuePool {
    pool: Arc<Mutex<InnerPool>>,
}

struct InnerPool {
    queues: Vec<Option<InnerQueue>>,
}

impl Default for ModificationQueuePool {
    fn default() -> Self {
        ModificationQueuePool::with_capacity(8)
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
        let (index, inner) = pool
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
            inner,
            index,
            pool: self.pool.clone(),
        }
    }
}

#[must_use = "dropping a modification queue discards the modifications within"]
pub struct ModificationQueue {
    inner: Option<InnerQueue>,
    index: usize,
    pool: Arc<Mutex<InnerPool>>,
}

impl Drop for ModificationQueue {
    fn drop(&mut self) {
        self.get_inner_mut().drain(..);
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool.queues[self.index] = self.inner.take();
    }
}

impl ModificationQueue {
    fn get_inner_mut(&mut self) -> &mut InnerQueue {
        self.inner
            .as_mut()
            .expect("modification queue should contain the inner queue at this point")
    }

    pub fn apply_range<R, W>(&mut self, world: &mut W, range: R)
    where
        R: RangeBounds<usize>,
        W: CanBeAppliedTo,
    {
        world.apply_range(self, range);
    }

    pub fn apply_all<W>(mut self, world: &mut W)
    where
        W: CanBeAppliedTo,
    {
        world.apply_all(self);
    }

    pub fn merge(&mut self, mut other: ModificationQueue) {
        self.get_inner_mut().extend(other.get_inner_mut().drain(..));
    }

    pub fn push<F>(&mut self, closure: F)
    where
        F: FnOnce(&mut World) + Send + Sync + 'static,
    {
        self.get_inner_mut().push(Box::new(closure))
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<Closure>
    where
        R: RangeBounds<usize>,
    {
        self.get_inner_mut().drain(range)
    }
}

impl Extend<Closure> for ModificationQueue {
    fn extend<T>(&mut self, iterator: T)
    where
        T: IntoIterator<Item = Closure>,
    {
        let inner = self.get_inner_mut();
        for closure in iterator {
            inner.push(closure);
        }
    }
}

pub trait CanBeAppliedTo {
    fn apply_range<R>(&mut self, queue: &mut ModificationQueue, range: R)
    where
        R: RangeBounds<usize>;

    fn apply_all(&mut self, queue: ModificationQueue);
}

impl CanBeAppliedTo for World {
    fn apply_range<R>(&mut self, queue: &mut ModificationQueue, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.apply_range(queue, range);
    }

    fn apply_all(&mut self, queue: ModificationQueue) {
        self.apply_all(queue);
    }
}

impl CanBeAppliedTo for WorldProxy<'_> {
    fn apply_range<R>(&mut self, queue: &mut ModificationQueue, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.apply_range(queue, range);
    }

    fn apply_all(&mut self, queue: ModificationQueue) {
        self.apply_all(queue);
    }
}

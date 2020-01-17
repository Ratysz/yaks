use hecs::World;
use resources::Resources;
use std::sync::{Arc, Mutex};

type Closure = Box<dyn FnOnce(&mut World, &mut Resources) + Send + Sync + 'static>;

type ModQueuePoolArc = Arc<Mutex<Vec<Option<ModQueueInner>>>>;

#[derive(Default)]
struct ModQueueInner {
    closures: Vec<Closure>,
    dirty: bool,
}

pub struct ModQueuePool {
    pool: ModQueuePoolArc,
}

impl Default for ModQueuePool {
    fn default() -> Self {
        ModQueuePool::with_capacity(4)
    }
}

impl ModQueuePool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let queues = std::iter::repeat_with(|| Some(Default::default()))
            .take(capacity)
            .collect();
        Self {
            pool: Arc::new(Mutex::new(queues)),
        }
    }

    pub fn get(&self) -> ModQueue {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        let (index, inner) = pool
            .iter_mut()
            .enumerate()
            .find(|(_, option)| {
                if let Some(inner) = option {
                    !inner.dirty
                } else {
                    false
                }
            })
            .map(|(index, option)| (index, option.take()))
            .unwrap_or_else(|| {
                pool.push(None);
                (pool.len() - 1, Some(Default::default()))
            });
        ModQueue {
            inner,
            index,
            pool: self.pool.clone(),
        }
    }

    pub fn apply_all(&self, world: &mut World, resources: &mut Resources) {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool.iter_mut().for_each(|option| {
            if let Some(ref mut inner) = option {
                inner
                    .closures
                    .drain(..)
                    .for_each(|closure| closure(world, resources));
                inner.dirty = false;
            }
        });
    }
}

pub struct ModQueue {
    inner: Option<ModQueueInner>,
    index: usize,
    pool: ModQueuePoolArc,
}

impl Drop for ModQueue {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool[self.index] = self.inner.take();
    }
}

impl ModQueue {
    fn get_inner_mut(&mut self, will_be_dirty: bool) -> &mut Vec<Closure> {
        self.inner
            .as_mut()
            .map(|inner| {
                inner.dirty = will_be_dirty;
                &mut inner.closures
            })
            .expect("modification queue should contain the inner queue at this point")
    }

    pub fn push<F>(&mut self, closure: F)
    where
        F: FnOnce(&mut World, &mut Resources) + Send + Sync + 'static,
    {
        self.get_inner_mut(true).push(Box::new(closure))
    }
}

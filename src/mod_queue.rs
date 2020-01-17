use hecs::World;
use resources::Resources;
use std::sync::{Arc, Mutex};

type Closure = Box<dyn FnOnce(&mut World, &mut Resources) + Send + Sync + 'static>;

type ModQueuePoolArc = Arc<Mutex<Vec<Option<Vec<Closure>>>>>;

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

    pub fn new_mod_queue(&self) -> ModQueue {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        let (index, closures) = pool
            .iter_mut()
            .enumerate()
            .find(|(_, option)| option.is_some())
            .map(|(index, option)| (index, option.take()))
            .unwrap_or_else(|| {
                pool.push(None);
                (pool.len() - 1, Some(Default::default()))
            });
        ModQueue {
            closures,
            index,
            pool: self.pool.clone(),
        }
    }

    pub fn apply_all(&self, world: &mut World, resources: &mut Resources) {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool.iter_mut().for_each(|option| {
            if let Some(ref mut closures) = option {
                closures
                    .drain(..)
                    .for_each(|closure| closure(world, resources));
            }
        });
    }
}

pub struct ModQueue {
    closures: Option<Vec<Closure>>,
    index: usize,
    pool: ModQueuePoolArc,
}

impl Drop for ModQueue {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        pool[self.index] = self.closures.take();
    }
}

impl ModQueue {
    fn closures_mut(&mut self) -> &mut Vec<Closure> {
        self.closures
            .as_mut()
            .expect("modification queue should contain the inner queue at this point")
    }

    pub fn push<F>(&mut self, closure: F)
    where
        F: FnOnce(&mut World, &mut Resources) + Send + Sync + 'static,
    {
        self.closures_mut().push(Box::new(closure))
    }
}

use std::sync::{Arc, Mutex};

use crate::World;

type Closure = Box<dyn FnOnce(&mut World) + Send + Sync + 'static>;

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
        ModQueuePool::with_capacity(8)
    }
}

impl ModQueuePool {
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

    pub fn flatten(&mut self) -> Option<ModQueue> {
        let mut pool = self.pool.lock().expect("mutexes should never be poisoned");
        let mut iterator = pool
            .iter_mut()
            .enumerate()
            .filter(|(_, option)| option.is_some());
        iterator.next().map(|(index, inner)| {
            let inner = inner.take().map(|mut inner| {
                inner.closures.extend(
                    iterator
                        .filter_map(|(_, inner)| {
                            inner
                                .as_mut()
                                .and_then(|inner| if inner.dirty { Some(inner) } else { None })
                        })
                        .flat_map(|inner| inner.closures.drain(..)),
                );
                inner
            });
            ModQueue {
                inner,
                index,
                pool: self.pool.clone(),
            }
        })
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
    fn get_inner_mut(&mut self) -> &mut Vec<Closure> {
        self.inner
            .as_mut()
            .map(|inner| {
                inner.dirty = true;
                &mut inner.closures
            })
            .expect("modification queue should contain the inner queue at this point")
    }

    pub fn push<F>(&mut self, closure: F)
    where
        F: FnOnce(&mut World) + Send + Sync + 'static,
    {
        self.get_inner_mut().push(Box::new(closure))
    }

    pub(crate) fn apply_all(&mut self, world: &mut World) {
        self.get_inner_mut()
            .drain(..)
            .for_each(|closure| closure(world));
    }
}

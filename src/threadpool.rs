use crossbeam::channel::{self, Receiver, Sender};
use hecs::{Entity, Fetch, Query, QueryBorrow};
use std::{
    mem,
    sync::atomic::{AtomicU32, Ordering},
    thread::{self, JoinHandle},
};

pub(crate) const DISCONNECTED: &str = "channel should not be disconnected at this point";

enum Message<'a> {
    Task(Box<dyn FnOnce() + Send + 'a>),
    Shutdown,
}

pub struct Threadpool {
    threads: Vec<JoinHandle<()>>,
    sender: Sender<Message<'static>>,
}

impl Threadpool {
    pub fn new(thread_count: usize) -> Self {
        let (sender, thread_receiver) = channel::unbounded();
        let mut threads = Vec::with_capacity(thread_count);

        threads.extend((0..thread_count).map(|_| {
            let thread_receiver = thread_receiver.clone();
            thread::Builder::new()
                .name("threadpool_worker".to_owned())
                .spawn(move || {
                    while let Ok(message) = thread_receiver.recv() {
                        match message {
                            Message::Task(closure) => closure(),
                            Message::Shutdown => break,
                        }
                    }
                })
                .expect("creating threads should always succeed")
        }));

        Self { threads, sender }
    }

    pub fn scope(&self) -> Scope {
        Scope::new(self)
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        for _ in 0..self.threads.len() {
            self.sender.send(Message::Shutdown).expect(DISCONNECTED);
        }
        while let Some(handle) = self.threads.pop() {
            drop(handle.join());
        }
    }
}

pub struct Scope<'scope> {
    pool: &'scope Threadpool,
    tasks: AtomicU32,
    sender: Sender<Result<(), ()>>,
    receiver: Receiver<Result<(), ()>>,
}

impl<'scope> Scope<'scope> {
    fn new(pool: &'scope Threadpool) -> Self {
        let (sender, receiver) = channel::unbounded();
        Self {
            pool,
            tasks: AtomicU32::new(0),
            sender,
            receiver,
        }
    }

    pub fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let sender = self.sender.clone();
        let closure = move || {
            let _unlocker = Unlocker(sender);
            closure();
        };
        let message = Message::Task(Box::new(closure));
        let message = unsafe {
            // Scope's Drop implementation ensures that the task does not outlive 'scope.
            mem::transmute(message)
        };
        self.pool.sender.send(message).expect(DISCONNECTED);
        self.tasks.fetch_add(1, Ordering::SeqCst);
    }

    pub fn scope(&self) -> Scope {
        self.pool.scope()
    }

    pub fn batch<'q, 'w, F, Q>(
        &self,
        query_borrow: &'q mut QueryBorrow<'w, Q>,
        batch_size: u32,
        for_each: F,
    ) where
        F: Fn(Entity, <<Q as Query>::Fetch as Fetch<'q>>::Item) + Send + Sync,
        Q: Query + Send + Sync + 'q,
    {
        query_borrow.iter_batched(batch_size).for_each(|batch| {
            self.execute(|| batch.for_each(|(entity, components)| for_each(entity, components)));
        });
    }
}

impl<'scope> Drop for Scope<'scope> {
    fn drop(&mut self) {
        let mut panicked = false;
        for _ in 0..*self.tasks.get_mut() {
            if self.receiver.recv().expect(DISCONNECTED).is_err() {
                panicked = true;
            }
        }
        if panicked {
            panic!("a worker thread has panicked");
        }
    }
}

struct Unlocker(Sender<Result<(), ()>>);

impl Drop for Unlocker {
    fn drop(&mut self) {
        if thread::panicking() {
            self.0.send(Err(())).expect(DISCONNECTED);
        } else {
            self.0.send(Ok(())).expect(DISCONNECTED);
        }
    }
}

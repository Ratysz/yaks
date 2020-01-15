pub trait Threadpool {
    fn execute<'a, F>(&mut self, closure: F)
    where
        F: FnOnce() + Send + 'a;
}

#[cfg(feature = "impl_scoped_threadpool")]
impl Threadpool for scoped_threadpool::Pool {
    fn execute<'a, F>(&mut self, closure: F)
    where
        F: FnOnce() + Send + 'a,
    {
        self.scoped(|scope| {
            scope.execute(closure);
        });
    }
}

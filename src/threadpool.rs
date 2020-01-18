pub trait Threadpool<'pool, 'scope, S>
where
    S: Scope<'pool, 'scope>,
{
    fn scope<F>(&'pool mut self, closure: F)
    where
        F: FnOnce(&S);
}

pub trait Scope<'pool, 'scope> {
    fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'scope;
}

#[cfg(feature = "impl_scoped_threadpool")]
impl<'pool, 'scope> Threadpool<'pool, 'scope, scoped_threadpool::Scope<'pool, 'scope>>
    for scoped_threadpool::Pool
{
    fn scope<F>(&'pool mut self, closure: F)
    where
        F: FnOnce(&scoped_threadpool::Scope<'pool, 'scope>),
    {
        self.scoped(closure);
    }
}

#[cfg(feature = "impl_scoped_threadpool")]
impl<'pool, 'scope> Scope<'pool, 'scope> for scoped_threadpool::Scope<'pool, 'scope> {
    fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        self.execute(closure);
    }
}

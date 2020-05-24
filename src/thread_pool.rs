#[cfg(not(feature = "parallel"))]
use std::marker::PhantomData;

#[cfg(feature = "parallel")]
type Scope<'scope> = rayon::Scope<'scope>;
#[cfg(not(feature = "parallel"))]
type Scope<'scope> = PhantomData<&'scope ()>;

pub trait ThreadPool {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope;
}

#[cfg(feature = "parallel")]
impl ThreadPool for &'_ rayon::ThreadPool {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope,
    {
        self.scope(closure);
    }
}

#[cfg(feature = "parallel")]
impl ThreadPool for () {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope,
    {
        rayon::scope(closure);
    }
}

#[cfg(feature = "parallel")]
impl ThreadPool for &'_ () {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope,
    {
        rayon::scope(closure);
    }
}

#[cfg(not(feature = "parallel"))]
impl ThreadPool for () {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope,
    {
        closure(&PhantomData);
    }
}

#[cfg(not(feature = "parallel"))]
impl ThreadPool for &'_ () {
    fn scope<'scope, F>(self, closure: F)
    where
        F: for<'s> FnOnce(&'s Scope<'scope>) + Send + 'scope,
    {
        closure(&PhantomData);
    }
}

use std::ops::{Deref, DerefMut};

use crate::{MarkerGet, Mut, Query, Ref};

pub trait Fetchable<Source> {
    type Fetched;

    fn fetch(source: Source) -> Self::Fetched;

    fn deref(fetched: &mut Self::Fetched) -> Self;
}

impl<Source, R> Fetchable<Source> for &'_ R
where
    Source: Copy,
    Ref<R>: MarkerGet<Source>,
    <Ref<R> as MarkerGet<Source>>::Fetched: Deref<Target = R>,
{
    type Fetched = <Ref<R> as MarkerGet<Source>>::Fetched;

    fn fetch(source: Source) -> Self::Fetched {
        <Ref<R> as MarkerGet<Source>>::fetch(source)
    }

    fn deref(fetched: &mut Self::Fetched) -> Self {
        unsafe { std::mem::transmute(&**fetched) }
    }
}

impl<Source, R> Fetchable<Source> for &'_ mut R
where
    Source: Copy,
    Mut<R>: MarkerGet<Source>,
    <Mut<R> as MarkerGet<Source>>::Fetched: DerefMut<Target = R>,
{
    type Fetched = <Mut<R> as MarkerGet<Source>>::Fetched;

    fn fetch(source: Source) -> Self::Fetched {
        <Mut<R> as MarkerGet<Source>>::fetch(source)
    }

    fn deref(fetched: &mut Self::Fetched) -> Self {
        unsafe { std::mem::transmute(&mut **fetched) }
    }
}

// TODO improve doc
/// Automatically implemented on all closures and functions that can be used
/// as systems in an executor. It shouldn't be implemented manually.
pub trait Run<Source, Marker, Resources, Queries> {
    /// Zero-cost wrapping function that executes the system.
    fn run(&mut self, world: &hecs::World, resources: Source);
}

pub struct SingleMarker;

pub struct TupleMarker;

macro_rules! impl_system {
    ((), ($($query:ident,)*)) => {
        impl<'closure, Closure, $($query,)*>
            Run<(), (), (), ($($query,)*)> for Closure
        where
            Closure: FnMut($(Query<$query>,)*) + Send + Sync + 'closure,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, _: ()) {
                self($(Query::<$query>::new(world),)*);
            }
        }
        // This allows using arbitrary Source for running systems with no resources,
        // but breaks inference of Marker for system.run(&world, ()).
        /*impl<'closure, Closure, Source, $($query,)*>
            Run<Source, (Source, ()), (), ($($query,)*)> for Closure
        where
            Closure: FnMut($(Query<$query>,)*) + Send + Sync + 'closure,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, _: Source) {
                self($(Query::<$query>::new(world),)*);
            }
        }*/
    };
    ($resource:ident, ($($query:ident,)*)) => {
        impl<'closure, Closure, $resource, $($query,)*>
            Run<$resource, SingleMarker, $resource, ($($query,)*)> for Closure
        where
            Closure: FnMut($resource, $(Query<$query>,)*) + Send + Sync + 'closure,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, $resource: $resource) {
                self($resource, $(Query::<$query>::new(world),)*);
            }
        }
        impl<'closure, Closure, Source, $resource, $($query,)*>
            Run<Source, (Source, SingleMarker), $resource, ($($query,)*)> for Closure
        where
            Closure: FnMut($resource, $(Query<$query>,)*) + Send + Sync + 'closure,
            $resource: Fetchable<Source>,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, resources: Source) {
                let mut $resource = $resource::fetch(resources);
                self($resource::deref(&mut $resource), $(Query::<$query>::new(world),)*);
            }
        }
    };
    (($($resource:ident,)*), ($($query:ident,)*)) => {
        impl<'closure, Closure, $($resource,)* $($query,)*>
            Run<($($resource,)*), TupleMarker, ($($resource,)*), ($($query,)*)> for Closure
        where
            Closure: FnMut($($resource,)* $(Query<$query>,)*) + Send + Sync + 'closure,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, ($($resource,)*): ($($resource,)*)) {
                self($($resource,)* $(Query::<$query>::new(world),)*);
            }
        }
        impl<'closure, Closure, Source, $($resource,)* $($query,)*>
            Run<Source, (Source, TupleMarker), ($($resource,)*), ($($query,)*)> for Closure
        where
            Closure: FnMut($($resource,)* $(Query<$query>,)*) + Send + Sync + 'closure,
            Source: Copy,
            $($resource: Fetchable<Source>,)*
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, resources: Source) {
                let ($(mut $resource,)*) = ($($resource::fetch(resources),)*);
                self($($resource::deref(&mut $resource),)* $(Query::<$query>::new(world),)*);
            }
        }
    };
}

impl_for_res_and_query_tuples!(impl_system);

#[test]
fn smoke_test() {
    let world = hecs::World::new();

    fn dummy_system() {}
    dummy_system.run(&world, ());

    let mut counter = 0i32;
    fn increment_system(value: &mut i32) {
        *value += 1;
    }
    increment_system.run(&world, &mut counter);
    assert_eq!(counter, 1);

    let increment = 3usize;
    fn sum_system(a: &mut i32, b: &usize) {
        *a += *b as i32;
    }
    sum_system.run(&world, (&mut counter, &increment));
    assert_eq!(counter, 4);
    sum_system.run(&world, (&mut counter, &increment));
    assert_eq!(counter, 7);
}

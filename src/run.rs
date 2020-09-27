use std::ops::{Deref, DerefMut};

use crate::{MarkerGet, Mut, Query, Ref};

pub trait MarkerGettable<Source> {
    type Intermediate;

    fn get(source: Source) -> Self::Intermediate;

    unsafe fn deref_to_self(fetched: &mut Self::Intermediate) -> Self;
}

impl<Source, R> MarkerGettable<Source> for &'_ R
where
    Source: Copy,
    Ref<R>: MarkerGet<Source>,
    <Ref<R> as MarkerGet<Source>>::Intermediate: Deref<Target = R>,
{
    type Intermediate = <Ref<R> as MarkerGet<Source>>::Intermediate;

    fn get(source: Source) -> Self::Intermediate {
        <Ref<R> as MarkerGet<Source>>::get(source)
    }

    unsafe fn deref_to_self(fetched: &mut Self::Intermediate) -> Self {
        std::mem::transmute(&**fetched)
    }
}

impl<Source, R> MarkerGettable<Source> for &'_ mut R
where
    Source: Copy,
    Mut<R>: MarkerGet<Source>,
    <Mut<R> as MarkerGet<Source>>::Intermediate: DerefMut<Target = R>,
{
    type Intermediate = <Mut<R> as MarkerGet<Source>>::Intermediate;

    fn get(source: Source) -> Self::Intermediate {
        <Mut<R> as MarkerGet<Source>>::get(source)
    }

    unsafe fn deref_to_self(fetched: &mut Self::Intermediate) -> Self {
        std::mem::transmute(&mut **fetched)
    }
}

// TODO add examples
/// Automatically implemented on all closures and functions that can be used
/// as systems in an executor. It's never required to be implemented manually.
pub trait Run<Source, Marker, Resources, Queries> {
    /// Zero-cost convenience function that calls the implementing function or closure.
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
            $resource: MarkerGettable<Source>,
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, resources: Source) {
                let mut $resource = $resource::get(resources);
                unsafe {
                    self(
                        $resource::deref_to_self(&mut $resource),
                        $(Query::<$query>::new(world),)*
                    );
                }
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
            $($resource: MarkerGettable<Source>,)*
            $($query: hecs::Query,)*
        {
            #[allow(non_snake_case, unused_variables)]
            fn run(&mut self, world: &hecs::World, resources: Source) {
                let ($(mut $resource,)*) = ($($resource::get(resources),)*);
                unsafe {
                    self(
                        $($resource::deref_to_self(&mut $resource),)*
                        $(Query::<$query>::new(world),)*
                    );
                }
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

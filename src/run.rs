use std::ops::{Deref, DerefMut};

use crate::{MarkerGet, Mut, QueryBundle, Ref};

// TODO improve doc
/// Automatically implemented on all closures and functions that can be used
/// as systems in an executor. It shouldn't be implemented manually.
pub trait System<'run, 'closure, Resources, Queries, Source, Marker> {
    /// Zero-cost wrapping function that executes the system.
    fn run(&'run mut self, world: &'run hecs::World, resources: Source);
}

impl<'run, 'closure, Closure, Resources, Queries>
    System<'run, 'closure, Resources, Queries, Resources, ()> for Closure
where
    Closure: FnMut(Resources, Queries) + Send + Sync + 'closure,
    Resources: Send + Sync,
    Queries: QueryBundle<'run>,
{
    fn run(&'run mut self, world: &'run hecs::World, resources: Resources) {
        self(resources, Queries::queries(world));
    }
}

pub trait Fetchable<Source> {
    type Fetched;

    fn fetch(source: Source) -> Self::Fetched;

    fn deref(fetched: &mut Self::Fetched) -> Self;
}

impl<Source, R0> Fetchable<Source> for &'_ R0
where
    Source: Copy,
    Ref<R0>: MarkerGet<Source>,
    <Ref<R0> as MarkerGet<Source>>::Fetched: Deref<Target = R0>,
{
    type Fetched = <Ref<R0> as MarkerGet<Source>>::Fetched;

    fn fetch(source: Source) -> Self::Fetched {
        <Ref<R0> as MarkerGet<Source>>::fetch(source)
    }

    fn deref(fetched: &mut Self::Fetched) -> Self {
        unsafe { std::mem::transmute(&**fetched) }
    }
}

impl<Source, R0> Fetchable<Source> for &'_ mut R0
where
    Source: Copy,
    Mut<R0>: MarkerGet<Source>,
    <Mut<R0> as MarkerGet<Source>>::Fetched: DerefMut<Target = R0>,
{
    type Fetched = <Mut<R0> as MarkerGet<Source>>::Fetched;

    fn fetch(source: Source) -> Self::Fetched {
        <Mut<R0> as MarkerGet<Source>>::fetch(source)
    }

    fn deref(fetched: &mut Self::Fetched) -> Self {
        unsafe { std::mem::transmute(&mut **fetched) }
    }
}

impl<'run, 'closure, Closure, A, Queries, Source>
    System<'run, 'closure, (A,), Queries, Source, (Source,)> for Closure
where
    Source: Copy,
    Closure: FnMut((A,), Queries) + 'closure,
    Closure: System<'run, 'closure, (A,), Queries, (A,), ()>,
    A: Fetchable<Source>,
    Queries: QueryBundle<'run>,
{
    fn run(&'run mut self, world: &'run hecs::World, resources: Source) {
        let mut a = A::fetch(resources);
        self.run(world, (A::deref(&mut a),));
    }
}

macro_rules! impl_system {
    ($($letter:ident),*) => {
        impl<'run, 'closure, Closure, $($letter),*, Queries, Source>
            System<'run, 'closure, ($($letter),*), Queries, Source, (Source, )> for Closure
        where
            Source: Copy,
            Closure: FnMut(($($letter),*), Queries) + 'closure,
            Closure: System<'run, 'closure, ($($letter),*), Queries, ($($letter),*), ()>,
            $($letter: Fetchable<Source>,)*
            Queries: QueryBundle<'run>,
        {
            #[allow(non_snake_case)]
            fn run(&'run mut self, world: &'run hecs::World, resources: Source) {
                let ($(mut $letter,)*) = ($($letter::fetch(resources),)*);
                self.run(world, ($($letter::deref(&mut $letter),)*));
            }
        }
    }
}

impl_for_tuples!(impl_system);

#[test]
fn smoke_test() {
    let world = hecs::World::new();

    fn dummy_system(_: (), _: ()) {}
    dummy_system.run(&world, ());

    let mut counter = 0i32;
    fn increment_system(value: &mut i32, _: ()) {
        *value += 1;
    }
    increment_system.run(&world, &mut counter);
    assert_eq!(counter, 1);

    let increment = 3usize;
    fn sum_system((a, b): (&mut i32, &usize), _: ()) {
        *a += *b as i32;
    }
    sum_system.run(&world, (&mut counter, &increment));
    assert_eq!(counter, 4);
    sum_system.run(&world, (&mut counter, &increment));
    assert_eq!(counter, 7);
}

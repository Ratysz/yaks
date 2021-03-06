use hecs::World;
use resources::{Ref, RefMut, Resource, Resources};

use crate::{Executor, QueryBundle, RefExtractor, System, SystemContext};

// TODO sprinkle this in doc examples

impl RefExtractor<&Resources> for () {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, _: &Resources) {
        executor.run(world, ());
    }
}

impl<R0> RefExtractor<&Resources> for (R0,)
where
    R0: Resource,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: &Resources) {
        let mut refs = resources
            .fetch::<&mut R0>()
            .unwrap_or_else(|error| panic!("{}", error));
        let derefs = (&mut *refs,);
        executor.run(world, derefs);
    }
}

macro_rules! impl_ref_extractor {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> RefExtractor<&Resources> for ($($letter,)*)
        where
            $($letter: Resource,)*
        {
            #[allow(non_snake_case)]
            fn extract_and_run(
                executor: &mut Executor<Self>,
                world: &World,
                resources: &Resources,
            ) {
                let ($(mut $letter,)*) = resources
                    .fetch::<($(&mut $letter, )*)>()
                    .unwrap_or_else(|error| panic!("{}", error));
                let derefs = ($(&mut *$letter,)*);
                executor.run(world, derefs);
            }
        }
    }
}

impl_for_tuples!(impl_ref_extractor);

pub trait Fetch<'a> {
    type Wrapped;

    fn fetch(resources: &'a Resources) -> Self::Wrapped;

    fn deref(wrapped: &mut Self::Wrapped) -> Self;
}

impl<'a, R0> Fetch<'a> for &'_ R0
where
    R0: Resource,
{
    type Wrapped = Ref<'a, R0>;

    fn fetch(resources: &'a Resources) -> Self::Wrapped {
        resources.get().unwrap_or_else(|error| panic!("{}", error))
    }

    fn deref(wrapped: &mut Self::Wrapped) -> Self {
        unsafe { std::mem::transmute(&**wrapped) }
    }
}

impl<'a, R0> Fetch<'a> for &'_ mut R0
where
    R0: Resource,
{
    type Wrapped = RefMut<'a, R0>;

    fn fetch(resources: &'a Resources) -> Self::Wrapped {
        resources
            .get_mut()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn deref(wrapped: &mut Self::Wrapped) -> Self {
        unsafe { std::mem::transmute(&mut **wrapped) }
    }
}

impl<'a, 'closure, Closure, Queries> System<'closure, (), Queries, &'a Resources, Resources>
    for Closure
where
    Closure: FnMut(SystemContext, (), Queries) + 'closure,
    Closure: System<'closure, (), Queries, (), ()>,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World, _: &'a Resources) {
        self.run(world, ());
    }
}

impl<'a, 'closure, Closure, A, Queries> System<'closure, A, Queries, &'a Resources, Resources>
    for Closure
where
    Closure: FnMut(SystemContext, A, Queries) + 'closure,
    Closure: System<'closure, A, Queries, A, ()>,
    for<'r0> A: Fetch<'r0>,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World, resources: &'a Resources) {
        let mut refs = A::fetch(resources);
        self.run(world, A::deref(&mut refs));
    }
}

macro_rules! impl_system {
    ($($letter:ident),*) => {
        impl<'a, 'closure, Closure, $($letter),*, Queries>
            System<'closure, ($($letter),*), Queries, &'a Resources, Resources> for Closure
        where
            Closure: FnMut(SystemContext, ($($letter),*), Queries) + 'closure,
            Closure: System<'closure, ($($letter),*), Queries, ($($letter),*), ()>,
            $(for<'r> $letter: Fetch<'r>,)*
            Queries: QueryBundle,
        {
            #[allow(non_snake_case)]
            fn run(&mut self, world: &World, resources: &'a Resources) {
                let ($(mut $letter,)*) = ($($letter::fetch(resources),)*);
                self.run(world, ($($letter::deref(&mut $letter),)*));
            }
        }
    }
}

impl_for_tuples!(impl_system);

#[test]
fn smoke_test() {
    use crate::Executor;
    let mut executor = Executor::<(f32, u32, u64)>::builder()
        .system(|_, _: (&mut f32, &u32), _: ()| {})
        .system(|_, _: (&mut f32, &u64), _: ()| {})
        .build();
    let world = hecs::World::new();

    let (mut a, mut b, mut c) = (1.0f32, 2u32, 3u64);
    executor.run(&world, (&mut a, &mut b, &mut c));

    let mut resources = resources::Resources::new();
    resources.insert(1.0f32);
    resources.insert(2u32);
    resources.insert(3u64);
    executor.run(&world, &resources);

    fn dummy_system(_: SystemContext, _: (), _: ()) {}
    dummy_system.run(&world, &resources);

    fn sum_system(_: SystemContext, (a, b): (&mut i32, &usize), _: ()) {
        *a += *b as i32;
    }
    resources.insert(3usize);
    resources.insert(1i32);
    sum_system.run(&world, &resources);
    assert_eq!(*resources.get::<i32>().unwrap(), 4);
    sum_system.run(&world, &resources);
    assert_eq!(*resources.get::<i32>().unwrap(), 7);
}

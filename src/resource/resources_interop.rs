use super::{MarkerGet, Mut, Ref};

// TODO sprinkle this in doc examples
use resources::{Resource, Resources};

impl<'a, T> MarkerGet<&'a Resources> for Ref<T>
where
    T: Resource,
{
    type Fetched = resources::Ref<'a, T>;

    fn fetch(source: &'a Resources) -> Self::Fetched {
        source.get().unwrap_or_else(|error| panic!("{}", error))
    }
}

impl<'a, T> MarkerGet<&'a Resources> for Mut<T>
where
    T: Resource,
{
    type Fetched = resources::RefMut<'a, T>;

    fn fetch(source: &'a Resources) -> Self::Fetched {
        source.get_mut().unwrap_or_else(|error| panic!("{}", error))
    }
}

#[test]
fn smoke_test() {
    use crate::{Executor, System, SystemContext};
    let mut executor = Executor::<(Mut<f32>, Ref<u32>, Ref<u64>)>::builder()
        .system(|_, _: (&mut f32, &u32), _: ()| {})
        .system(|_, _: (&mut f32, &u64), _: ()| {})
        .system(|_, _: (), _: ()| {})
        .build();
    let world = hecs::World::new();

    let (mut a, b, c) = (1.0f32, 2u32, 3u64);
    executor.run(&world, (&mut a, &b, &c));

    let mut resources = resources::Resources::new();
    resources.insert(1.0f32);
    resources.insert(2u32);
    resources.insert(3u64);
    executor.run(&world, &resources);

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

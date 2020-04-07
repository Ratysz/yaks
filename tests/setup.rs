use yaks::{ModQueuePool, Resources, World};

pub struct Res1(pub usize);

pub struct Res2(pub f32);

pub struct Comp1(pub usize);

pub struct Comp2(pub f32);

pub struct Comp3(pub &'static str);

pub fn setup() -> (World, Resources, ModQueuePool) {
    let mut world = World::new();
    world.spawn((Comp1(1), Comp2(0.0)));
    world.spawn((Comp1(0), Comp2(1.0)));
    world.spawn((Comp1(1), Comp2(2.0), Comp3("one")));
    world.spawn((Comp1(2), Comp2(1.0), Comp3("two")));
    world.spawn((Comp1(1), Comp3("one")));
    world.spawn((Comp2(1.0), Comp3("two")));
    let mut resources = Resources::new();
    resources.insert(Res1(0));
    resources.insert(Res2(0.0));
    (world, resources, ModQueuePool::new())
}

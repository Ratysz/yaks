use hecs::World;

use crate::{QueryBundle, SystemContext};

// TODO improve doc
/// Automatically implemented on all closures and functions than
/// can be used as systems in an executor.
pub trait System<'closure, Resources, Queries, RefSource> {
    /// Zero-cost wrapping function that executes the system.
    fn run(&mut self, world: &World, resources: RefSource);
}

impl<'closure, Closure, Resources, Queries> System<'closure, Resources, Queries, Resources>
    for Closure
where
    Closure: FnMut(SystemContext, Resources, Queries) + Send + Sync + 'closure,
    Resources: Send + Sync,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World, resources: Resources) {
        self(world.into(), resources, Queries::markers());
    }
}

#[test]
fn test() {
    let world = hecs::World::new();

    fn hello_system(_: SystemContext, _: (), _: ()) {
        println!("Hello!")
    }
    hello_system.run(&world, ());

    let mut counter = 0i32;
    fn increment_system(_: SystemContext, value: &mut i32, _: ()) {
        *value += 1;
    }
    increment_system.run(&world, &mut counter);
    assert_eq!(counter, 1);

    let increment = 3usize;
    fn sum_system(_: SystemContext, (a, b): (&mut i32, &usize), _: ()) {
        *a += *b as i32;
    }
    sum_system.run(&world, (&mut counter, &increment));
    assert_eq!(counter, 4);
}

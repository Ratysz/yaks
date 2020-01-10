use secs::{System, World};

struct Res1(usize);

struct Res2(usize);

struct Res3(usize);

struct Comp1(usize);

struct Comp2(usize);

struct Comp3(usize);

struct Comp4(usize);

#[test]
fn basic() {
    let mut world = World::new();
    System::builder()
        .build(|world, _, _| {
            world.add_resource(Res1(0));
        })
        .run(&mut world);
    assert!(world.contains_resource::<Res1>())
}

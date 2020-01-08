use secs::{Executor, System, SystemHandle, World};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct Handle(usize);

impl SystemHandle for Handle {}

struct Res1(usize);

struct Res2(usize);

struct Res3(usize);

struct Comp1(usize);

struct Comp2(usize);

struct Comp3(usize);

struct Comp4(usize);

#[test]
fn single_no_handle() {
    let mut world = World::new();
    world.add_resource(Res1(0));
    let mut executor = Executor::<()>::new().with(System::builder().resource::<&mut Res1>().build(
        move |_, mut resource, _| {
            resource.0 += 1;
        },
    ));
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn non_unique_system_handle() {
    let mut executor = Executor::<Handle>::new();
    let result = executor.add_with_handle(Handle(0), System::builder().build(|_, _, _| {}));
    assert!(result.is_ok());
    let result = executor.add_with_handle(Handle(0), System::builder().build(|_, _, _| {}));
    assert!(result.is_err());
}

#[test]
fn single() {
    let mut world = World::new();
    world.add_resource(Res1(0));
    let mut executor = Executor::<Handle>::new().with_handle(
        Handle(0),
        System::builder()
            .resource::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn single_inactive() {
    let mut world = World::new();
    world.add_resource(Res1(0));
    let mut executor = Executor::<Handle>::new().with_handle_deactivated(
        Handle(0),
        System::builder()
            .resource::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 0);
}

#[test]
fn single_late_activation() {
    let mut world = World::new();
    world.add_resource(Res1(0));
    let mut executor = Executor::<Handle>::new().with_handle_deactivated(
        Handle(0),
        System::builder()
            .resource::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 0);
    executor.set_active(&Handle(0), true);
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn multiple() {
    let mut world = World::new();
    let mut executor = Executor::<()>::new().with(System::builder().build(|_, _, _| {}));
}

use secs::{Executor, QueryEffector, System, SystemHandle, World};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct Handle(usize);

impl SystemHandle for Handle {}

struct Res1(usize);

struct Res2(f32);

struct Comp1(usize);

struct Comp2(f32);

struct Comp3(&'static str);

fn setup_world() -> World {
    let mut world = World::new();
    world.add_resource(Res1(0));
    world.add_resource(Res2(0.0));
    world.spawn((Comp1(1), Comp2(0.0)));
    world.spawn((Comp1(0), Comp2(1.0)));
    world.spawn((Comp1(1), Comp2(1.0), Comp3("one")));
    world.spawn((Comp1(1), Comp2(1.0), Comp3("two")));
    world.spawn((Comp1(1), Comp3("one")));
    world.spawn((Comp2(1.0), Comp3("two")));
    world
}

#[test]
fn mod_queue_entity_spawn_despawn() {
    let mut world = setup_world();
    let query = QueryEffector::<(&Comp1, &Comp2, &Comp3)>::new();
    System::builder()
        .build(move |world, _, _| {
            assert_eq!(world.query_by(query).iter().collect::<Vec<_>>().len(), 2);
            let mut queue = world.new_mod_queue();
            queue.push(|world| {
                world.spawn((Comp1(6), Comp2(3.0), Comp3("NaN")));
            });
        })
        .run_and_flush(&mut world);
    let entities = world
        .query_by(query)
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(entities.len(), 3);
    let entity = entities[0];
    System::builder()
        .build(move |world, _, _| {
            assert_eq!(world.query_by(query).iter().collect::<Vec<_>>().len(), 3);
            let mut queue = world.new_mod_queue();
            queue.push(move |world| {
                assert!(world.despawn(entity).is_ok());
            });
        })
        .run_and_flush(&mut world);
    assert_eq!(world.query_by(query).iter().collect::<Vec<_>>().len(), 2);
}

#[test]
fn mod_queue_resource_add_remove() {
    let mut world = setup_world();
    assert!(world.contains_resource::<Res1>());
    System::builder()
        .build(|world, _, _| {
            let mut queue = world.new_mod_queue();
            queue.push(|world| {
                assert!(world.remove_resource::<Res1>().is_ok());
            });
        })
        .run_and_flush(&mut world);
    assert!(!world.contains_resource::<Res1>());
    System::builder()
        .build(|world, _, _| {
            let mut queue = world.new_mod_queue();
            queue.push(|world| {
                world.add_resource(Res1(1));
            });
        })
        .run_and_flush(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn mod_queue_manual_flushing() {
    let mut world = setup_world();
    let mut system_2 = System::builder().build(|world, _, _| {
        let mut queue = world.new_mod_queue();
        queue.push(|world| assert!(world.remove_resource::<Res1>().is_ok()));
    });
    let mut system_1 = System::builder()
        .resources::<&mut Res1>()
        .build(|_, mut resource, _| {
            resource.0 = 1;
        });
    assert!(world.contains_resource::<Res1>());
    system_1.run(&world);
    system_2.run(&world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
    world.flush_mod_queues();
    assert!(!world.contains_resource::<Res1>());
}

#[test]
fn executor_single_no_handle() {
    let mut world = setup_world();
    let mut executor =
        Executor::<()>::new().with(System::builder().resources::<&mut Res1>().build(
            move |_, mut resource, _| {
                resource.0 += 1;
            },
        ));
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn executor_non_unique_system_handle() {
    let mut executor = Executor::<Handle>::new();
    let result = executor.add_with_handle(Handle(0), System::builder().build(|_, _, _| {}));
    assert!(result.is_ok());
    let result = executor.add_with_handle(Handle(0), System::builder().build(|_, _, _| {}));
    assert!(result.is_err());
}

#[test]
fn executor_single() {
    let mut world = setup_world();
    let mut executor = Executor::<Handle>::new().with_handle(
        Handle(0),
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

#[test]
fn executor_single_inactive() {
    let mut world = setup_world();
    let mut executor = Executor::<Handle>::new().with_handle_deactivated(
        Handle(0),
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 0);
}

#[test]
fn executor_single_late_activation() {
    let mut world = setup_world();
    let mut executor = Executor::<Handle>::new().with_handle_deactivated(
        Handle(0),
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 0);
    assert!(executor.set_active(&Handle(0), true).is_ok());
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
}

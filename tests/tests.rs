use secs::{Executor, System, World};

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
#[should_panic]
fn system_invalid_resources() {
    let mut world = setup_world();
    System::builder()
        .resources::<(&Res1, &mut Res1)>()
        .build(|_, _, _| ())
        .run_and_flush(&mut world);
}

#[test]
fn mod_queue_entity_spawn_despawn() {
    let mut world = setup_world();
    type Query<'a> = (&'a Comp1, &'a Comp2, &'a Comp3);
    System::builder()
        .query::<Query>()
        .build(move |world, _, query| {
            assert_eq!(query.query(world).iter().collect::<Vec<_>>().len(), 2);
            let mut queue = world.new_mod_queue();
            queue.push(|world| {
                world.spawn((Comp1(6), Comp2(3.0), Comp3("NaN")));
            });
        })
        .run_and_flush(&mut world);
    let entities = world
        .query::<Query>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(entities.len(), 3);
    let entity = entities[0];
    System::builder()
        .query::<Query>()
        .build(move |world, _, query| {
            assert_eq!(query.query(world).iter().collect::<Vec<_>>().len(), 3);
            let mut queue = world.new_mod_queue();
            queue.push(move |world| {
                assert!(world.despawn(entity).is_ok());
            });
        })
        .run_and_flush(&mut world);
    assert_eq!(world.query::<Query>().iter().collect::<Vec<_>>().len(), 2);
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
    let mut executor = Executor::<usize>::new();
    let option = executor.add_with_handle(0, System::builder().build(|_, _, _| {}));
    assert!(option.is_none());
    let option = executor.add_with_handle(0, System::builder().build(|_, _, _| {}));
    assert!(option.is_some());
}

#[test]
fn executor_single() {
    let mut world = setup_world();
    let mut executor = Executor::<usize>::new().with_handle(
        0,
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
fn executor_single_handle() {
    let mut world = setup_world();
    let mut executor = Executor::<usize>::new().with_handle(
        0,
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    );
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
    assert!(executor.is_active(&0).unwrap());
    assert!(executor.set_active(&0, false).is_ok());
    assert!(!executor.is_active(&0).unwrap());
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 1);
    assert!(executor.set_active(&0, true).is_ok());
    executor.run(&mut world);
    assert_eq!(world.fetch::<&Res1>().0, 2);
    assert!(executor.is_active(&2).is_err())
}

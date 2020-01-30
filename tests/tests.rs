use yaks::{ModQueuePool, Resources, System, World};

struct Res1(usize);

struct Res2(f32);

struct Comp1(usize);

struct Comp2(f32);

struct Comp3(&'static str);

fn setup() -> (World, Resources, ModQueuePool) {
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

#[test]
#[should_panic]
fn system_invalid_resources() {
    let (world, resources, mod_queues) = setup();
    System::builder()
        .resources::<(&Res1, &mut Res1)>()
        .build(|_, _, _| ())
        .run(&world, &resources, &mod_queues);
}

#[test]
fn mod_queue_late_flushing() {
    let (mut world, mut resources, mod_queues) = setup();
    let mut system_1 = System::builder()
        .resources::<&mut Res1>()
        .build(|_, mut resource, _| {
            resource.0 = 1;
        });
    let mut system_2 = System::builder().build(|context, _, _| {
        context
            .new_mod_queue()
            .push(|_, resources| assert!(resources.remove::<Res1>().is_some()));
    });
    assert!(resources.contains::<Res1>());
    system_1.run(&world, &resources, &mod_queues);
    system_2.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    mod_queues.apply_all(&mut world, &mut resources);
    assert!(!resources.contains::<Res1>());
}

#[test]
fn mod_queue_entity_spawn_despawn() {
    let (mut world, mut resources, mod_queues) = setup();
    type Query<'a> = (&'a Comp1, &'a Comp2, &'a Comp3);
    System::builder()
        .query::<Query>()
        .build(|context, _, query| {
            assert_eq!(context.query(query).iter().collect::<Vec<_>>().len(), 2);
            context.new_mod_queue().push(|world, _| {
                world.spawn((Comp1(6), Comp2(3.0), Comp3("NaN")));
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    let entities = world
        .query::<Query>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(entities.len(), 3);
    let entity = entities[0];
    System::builder()
        .query::<Query>()
        .build(move |context, _, query| {
            assert_eq!(context.query(query).iter().collect::<Vec<_>>().len(), 3);
            context.new_mod_queue().push(move |world, _| {
                assert!(world.despawn(entity).is_ok());
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    assert_eq!(world.query::<Query>().iter().collect::<Vec<_>>().len(), 2);
}

#[test]
fn mod_queue_resource_add_remove() {
    let (mut world, mut resources, mod_queues) = setup();
    assert!(resources.contains::<Res1>());
    System::builder()
        .build(|context, _, _| {
            context.new_mod_queue().push(|_, resources| {
                assert!(resources.remove::<Res1>().is_some());
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    assert!(!resources.contains::<Res1>());
    System::builder()
        .build(|context, _, _| {
            context.new_mod_queue().push(|_, resources| {
                resources.insert(Res1(1));
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
}

#[cfg(feature = "parallel")]
#[test]
#[should_panic(expected = "a worker thread has panicked")]
fn threadpool_subscope_panic() {
    let threadpool = yaks::Threadpool::new(4);
    let scope = threadpool.scope();
    let subscope = scope.scope();
    scope.execute(move || {
        subscope.execute(|| panic!());
    });
}

#[cfg(feature = "parallel")]
#[test]
fn batch() {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use yaks::Threadpool;
    let (world, _, _) = setup();
    let threadpool = Threadpool::new(4);
    let scope = threadpool.scope();
    let time = Instant::now();
    scope.batch(
        &mut world.query::<(&Comp1, &mut Comp3)>(),
        1,
        |(_, (comp1, mut comp3))| {
            thread::sleep(Duration::from_millis(25));
            let _value = comp1.0 as f32;
            comp3.0 = "test";
        },
    );
    drop(scope);
    assert!(time.elapsed() < Duration::from_millis(100));
}

#[cfg(feature = "parallel")]
#[test]
fn batch_system() {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use yaks::Threadpool;
    let (world, resources, mod_queues) = setup();
    let mut system =
        System::builder()
            .query::<(&Comp1, &mut Comp3)>()
            .build(|context, _, query| {
                context.batch(&mut context.query(query), 1, |(_, (comp1, mut comp3))| {
                    thread::sleep(Duration::from_millis(25));
                    let _value = comp1.0 as f32;
                    comp3.0 = "test";
                });
            });
    let threadpool = Threadpool::new(4);
    let scope = threadpool.scope();
    let time = Instant::now();
    system.run_with_scope(&world, &resources, &mod_queues, &scope);
    drop(scope);
    assert!(time.elapsed() < Duration::from_millis(75));
    for (_, (_, comp3)) in world.query::<(&Comp1, &Comp3)>().iter() {
        assert_eq!(comp3.0, "test");
    }
    System::builder()
        .query::<(&Comp1, &mut Comp3)>()
        .build(|context, _, query| {
            for (_, (_, mut comp3)) in context.query(query).iter() {
                comp3.0 = "_";
            }
        })
        .run(&world, &resources, &mod_queues);
    for (_, (_, comp3)) in world.query::<(&Comp1, &Comp3)>().iter() {
        assert_eq!(comp3.0, "_");
    }
    let time = Instant::now();
    system.run(&world, &resources, &mod_queues);
    assert!(time.elapsed() > Duration::from_millis(75));
    for (_, (_, comp3)) in world.query::<(&Comp1, &Comp3)>().iter() {
        assert_eq!(comp3.0, "test");
    }
}

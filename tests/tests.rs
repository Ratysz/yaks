#[cfg(feature = "parallel")]
use yaks::ParallelExecutor;
use yaks::{Executor, ModQueuePool, Resources, System, World};

struct Res1(usize);

struct Res2(f32);

struct Comp1(usize);

struct Comp2(f32);

struct Comp3(&'static str);

fn setup() -> (World, Resources, ModQueuePool) {
    let mut world = World::new();
    world.spawn((Comp1(1), Comp2(0.0)));
    world.spawn((Comp1(0), Comp2(1.0)));
    world.spawn((Comp1(1), Comp2(1.0), Comp3("one")));
    world.spawn((Comp1(1), Comp2(1.0), Comp3("two")));
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
fn mod_queue_entity_spawn_despawn() {
    let (mut world, mut resources, mod_queues) = setup();
    type Query<'a> = (&'a Comp1, &'a Comp2, &'a Comp3);
    System::builder()
        .query::<Query>()
        .build(|facade, _, query| {
            assert_eq!(facade.query(query).iter().collect::<Vec<_>>().len(), 2);
            facade.new_mod_queue().push(|world, _| {
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
        .build(move |facade, _, query| {
            assert_eq!(facade.query(query).iter().collect::<Vec<_>>().len(), 3);
            facade.new_mod_queue().push(move |world, _| {
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
        .build(|facade, _, _| {
            facade.new_mod_queue().push(|_, resources| {
                assert!(resources.remove::<Res1>().is_some());
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    assert!(!resources.contains::<Res1>());
    System::builder()
        .build(|facade, _, _| {
            facade.new_mod_queue().push(|_, resources| {
                resources.insert(Res1(1));
            });
        })
        .run(&world, &resources, &mod_queues);
    mod_queues.apply_all(&mut world, &mut resources);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
}

#[test]
fn mod_queue_manual_flushing() {
    let (mut world, mut resources, mod_queues) = setup();
    let mut system_2 = System::builder().build(|facade, _, _| {
        facade
            .new_mod_queue()
            .push(|_, resources| assert!(resources.remove::<Res1>().is_some()));
    });
    let mut system_1 = System::builder()
        .resources::<&mut Res1>()
        .build(|_, mut resource, _| {
            resource.0 = 1;
        });
    assert!(resources.contains::<Res1>());
    system_1.run(&world, &resources, &mod_queues);
    system_2.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    mod_queues.apply_all(&mut world, &mut resources);
    assert!(!resources.contains::<Res1>());
}

#[test]
fn executor_single_no_handle() {
    let (world, resources, mod_queues) = setup();
    let mut executor =
        Executor::<()>::new().with(System::builder().resources::<&mut Res1>().build(
            move |_, mut resource, _| {
                resource.0 += 1;
            },
        ));
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
}

#[test]
fn executor_non_unique_system_handle() {
    let mut executor = Executor::<usize>::new();
    let option = executor.add((0, System::builder().build(|_, _, _| {})));
    assert!(option.is_none());
    let option = executor.add((0, System::builder().build(|_, _, _| {})));
    assert!(option.is_some());
}

#[test]
fn executor_single() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<usize>::new().with((
        0,
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    ));
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
}

#[test]
fn executor_single_handle() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<usize>::new().with((
        0,
        System::builder()
            .resources::<&mut Res1>()
            .build(move |_, mut resource, _| {
                resource.0 += 1;
            }),
    ));
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert!(executor.is_active(&0).unwrap());
    assert!(executor.set_active(&0, false).is_ok());
    assert!(!executor.is_active(&0).unwrap());
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert!(executor.set_active(&0, true).is_ok());
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 2);
    assert!(executor.is_active(&2).is_err())
}

#[cfg(feature = "parallel")]
#[test]
fn parallel_executor_same_resource_borrows() {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    let (world, resources, mod_queues) = setup();
    let mut executor = ParallelExecutor::<()>::new();
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    executor.add(
        System::builder()
            .resources::<&mut Res1>()
            .build(|_, mut res, _| {
                res.0 += 1;
                thread::sleep(Duration::from_millis(100));
            }),
    );
    executor.add(
        System::builder()
            .resources::<&mut Res1>()
            .build(|_, mut res, _| {
                res.0 += 1;
                thread::sleep(Duration::from_millis(100));
            }),
    );
    executor.run_parallel(&world, &resources, &mod_queues, &mut threadpool);
    assert!(time.elapsed() > Duration::from_millis(200));
    assert_eq!(resources.get::<Res1>().unwrap().0, 2);
}

#[cfg(feature = "parallel")]
#[test]
fn parallel_executor_disjoint_resource_borrows() {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    let (world, resources, mod_queues) = setup();
    let mut executor = ParallelExecutor::<()>::new();
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    executor.add(
        System::builder()
            .resources::<&mut Res1>()
            .build(|_, mut res, _| {
                res.0 += 1;
                thread::sleep(Duration::from_millis(100));
            }),
    );
    executor.add(
        System::builder()
            .resources::<&mut Res2>()
            .build(|_, mut res, _| {
                res.0 += 1.0;
                thread::sleep(Duration::from_millis(100));
            }),
    );
    executor.run_parallel(&world, &resources, &mod_queues, &mut threadpool);
    assert!(time.elapsed() < Duration::from_millis(200));
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert_eq!(resources.get::<Res2>().unwrap().0, 1.0);
}

#[cfg(feature = "parallel")]
#[test]
#[should_panic]
fn parallel_executor_invalid_resource_borrows() {
    let (world, resources, mod_queues) = setup();
    let mut executor = ParallelExecutor::<()>::new();
    let mut threadpool = scoped_threadpool::Pool::new(4);
    executor.add(System::builder().build(|facade, _, _| {
        let mut borrow = facade.resources.get_mut::<Res1>().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        borrow.0 += 1;
    }));
    executor.add(System::builder().build(|facade, _, _| {
        let mut borrow = facade.resources.get_mut::<Res1>().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        borrow.0 += 1;
    }));
    executor.run_parallel(&world, &resources, &mod_queues, &mut threadpool);
}

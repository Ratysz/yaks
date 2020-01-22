#[cfg(feature = "parallel")]
use std::{
    thread,
    time::{Duration, Instant},
};

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
    world.spawn((Comp1(1), Comp2(2.0), Comp3("one")));
    world.spawn((Comp1(2), Comp2(1.0), Comp3("two")));
    world.spawn((Comp1(1), Comp3("one")));
    world.spawn((Comp2(1.0), Comp3("two")));
    let mut resources = Resources::new();
    resources.insert(Res1(0));
    resources.insert(Res2(0.0));
    (world, resources, ModQueuePool::new())
}

#[cfg(feature = "parallel")]
#[test]
fn hard_dependency() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<usize>::new()
        .with((
            0,
            System::builder().build(|_, _, _| {
                thread::sleep(Duration::from_millis(100));
            }),
        ))
        .with((
            1,
            vec![0],
            System::builder().build(|_, _, _| {
                thread::sleep(Duration::from_millis(100));
            }),
        ));
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert!(time.elapsed() > Duration::from_millis(200));
}

#[cfg(feature = "parallel")]
#[test]
fn valid_resource_borrows() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.resources.get_mut::<Res1>().unwrap();
            thread::sleep(Duration::from_millis(100));
            borrow.0 += 1;
        }))
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.resources.get_mut::<Res2>().unwrap();
            thread::sleep(Duration::from_millis(100));
            borrow.0 += 1.0;
        }));
    let mut threadpool = scoped_threadpool::Pool::new(4);
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert_eq!(resources.get::<Res2>().unwrap().0, 1.0);
}

#[cfg(feature = "parallel")]
#[test]
#[should_panic]
fn invalid_resource_borrows() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.resources.get_mut::<Res1>().unwrap();
            thread::sleep(Duration::from_millis(100));
            borrow.0 += 1;
        }))
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.resources.get_mut::<Res1>().unwrap();
            thread::sleep(Duration::from_millis(100));
            borrow.0 += 1;
        }));
    let mut threadpool = scoped_threadpool::Pool::new(4);
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
}

#[cfg(feature = "parallel")]
#[test]
fn same_resource_borrows() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(
            System::builder()
                .resources::<&mut Res1>()
                .build(|_, mut res, _| {
                    res.0 += 1;
                    thread::sleep(Duration::from_millis(100));
                }),
        )
        .with(
            System::builder()
                .resources::<&mut Res1>()
                .build(|_, mut res, _| {
                    res.0 += 1;
                    thread::sleep(Duration::from_millis(100));
                }),
        );
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert!(time.elapsed() > Duration::from_millis(200));
    assert_eq!(resources.get::<Res1>().unwrap().0, 2);
}

#[cfg(feature = "parallel")]
#[test]
fn disjoint_resource_borrows() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(
            System::builder()
                .resources::<&mut Res1>()
                .build(|_, mut res, _| {
                    res.0 += 1;
                    thread::sleep(Duration::from_millis(100));
                }),
        )
        .with(
            System::builder()
                .resources::<&mut Res2>()
                .build(|_, mut res, _| {
                    res.0 += 1.0;
                    thread::sleep(Duration::from_millis(100));
                }),
        );
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert!(time.elapsed() < Duration::from_millis(200));
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert_eq!(resources.get::<Res2>().unwrap().0, 1.0);
}

#[cfg(feature = "parallel")]
#[test]
fn same_queries() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(
            System::builder()
                .query::<(&Comp1, &Comp2)>()
                .build(|facade, _, query| {
                    let mut borrow = facade.query(query);
                    for (_, (comp1, comp2)) in borrow.iter() {
                        thread::sleep(Duration::from_millis(25));
                        let _value = comp1.0 as f32 + comp2.0;
                    }
                }),
        )
        .with(
            System::builder()
                .query::<(&Comp1, &mut Comp2)>()
                .build(|facade, _, query| {
                    let mut borrow = facade.query(query);
                    for (_, (comp1, mut comp2)) in borrow.iter() {
                        thread::sleep(Duration::from_millis(25));
                        comp2.0 = comp1.0 as f32;
                    }
                }),
        );
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert!(time.elapsed() > Duration::from_millis(200));
    for (_, (comp1, comp2)) in world.query::<(&Comp1, &Comp2)>().iter() {
        assert_eq!(comp1.0 as f32, comp2.0);
    }
}

#[cfg(feature = "parallel")]
#[test]
fn disjoint_queries() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(
            System::builder()
                .query::<(&Comp1, &Comp2)>()
                .build(|facade, _, query| {
                    let mut borrow = facade.query(query);
                    for (_, (comp1, comp2)) in borrow.iter() {
                        thread::sleep(Duration::from_millis(25));
                        let _value = comp1.0 as f32 + comp2.0;
                    }
                }),
        )
        .with(
            System::builder()
                .query::<(&Comp1, &mut Comp3)>()
                .build(|facade, _, query| {
                    let mut borrow = facade.query(query);
                    for (_, (comp1, mut comp3)) in borrow.iter() {
                        thread::sleep(Duration::from_millis(25));
                        let _value = comp1.0 as f32;
                        comp3.0 = "test";
                    }
                }),
        );
    let mut threadpool = scoped_threadpool::Pool::new(4);
    let time = Instant::now();
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, (_, comp3)) in world.query::<(&Comp1, &Comp3)>().iter() {
        assert_eq!(comp3.0, "test");
    }
}

#[cfg(feature = "parallel")]
#[test]
fn valid_queries() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.world.query::<(&Comp1, &Comp2)>();
            for (_, (comp1, comp2)) in borrow.iter() {
                thread::sleep(Duration::from_millis(25));
                let _value = comp1.0 as f32 + comp2.0;
            }
        }))
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.world.query::<(&Comp1, &mut Comp3)>();
            for (_, (comp1, mut comp3)) in borrow.iter() {
                thread::sleep(Duration::from_millis(25));
                let _value = comp1.0 as f32;
                comp3.0 = "test";
            }
        }));
    let mut threadpool = scoped_threadpool::Pool::new(4);
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
    for (_, (_, comp3)) in world.query::<(&Comp1, &Comp3)>().iter() {
        assert_eq!(comp3.0, "test");
    }
}

#[cfg(feature = "parallel")]
#[test]
#[should_panic]
fn invalid_queries() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::new()
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.world.query::<(&Comp1, &Comp2)>();
            for (_, (comp1, comp2)) in borrow.iter() {
                thread::sleep(Duration::from_millis(25));
                let _value = comp1.0 as f32 + comp2.0;
            }
        }))
        .with(System::builder().build(|facade, _, _| {
            let mut borrow = facade.world.query::<(&Comp1, &mut Comp2)>();
            for (_, (comp1, mut comp2)) in borrow.iter() {
                thread::sleep(Duration::from_millis(25));
                comp2.0 = comp1.0 as f32;
            }
        }));
    let mut threadpool = scoped_threadpool::Pool::new(4);
    threadpool.scoped(|scope| {
        executor.run_parallel(&world, &resources, &mod_queues, scope);
    });
}

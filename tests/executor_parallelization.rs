use hecs::World;
use std::{
    thread,
    time::{Duration, Instant},
};
use yaks::{Executor, QueryMarker, SystemContext};

struct A(usize);
struct B(usize);
struct C(usize);

fn execute<F>(closure: F)
where
    F: FnOnce() + Send,
{
    #[cfg(feature = "parallel")]
    rayon::ThreadPoolBuilder::new()
        .build()
        .unwrap()
        .install(closure);
    #[cfg(not(feature = "parallel"))]
    closure();
}

fn sleep_millis(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn sleep_system(millis: u64) -> impl Fn(SystemContext, (), ()) {
    move |_, _: (), _: ()| {
        sleep_millis(millis);
    }
}

#[test]
fn dependencies_single() {
    let world = World::new();
    let mut executor = Executor::<()>::builder()
        .system_with_handle(sleep_system(100), 0)
        .system_with_handle_and_deps(sleep_system(100), 1, vec![0])
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, ()));
    assert!(time.elapsed() > Duration::from_millis(200));
}

#[test]
fn dependencies_several() {
    let world = World::new();
    let mut executor = Executor::<()>::builder()
        .system_with_handle(sleep_system(50), 0)
        .system_with_handle(sleep_system(50), 1)
        .system_with_handle(sleep_system(50), 2)
        .system_with_deps(sleep_system(50), vec![0, 1, 2])
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, ()));
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    {
        let elapsed = time.elapsed();
        assert!(elapsed < Duration::from_millis(150));
        assert!(elapsed > Duration::from_millis(100));
    }
}

#[test]
fn dependencies_chain() {
    let world = World::new();
    let mut executor = Executor::<()>::builder()
        .system_with_handle(sleep_system(50), 0)
        .system_with_handle_and_deps(sleep_system(50), 1, vec![0])
        .system_with_handle_and_deps(sleep_system(50), 2, vec![1])
        .system_with_deps(sleep_system(50), vec![2])
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, ()));
    assert!(time.elapsed() > Duration::from_millis(200));
}

#[test]
fn dependencies_fully_constrained() {
    let world = World::new();
    let mut executor = Executor::<()>::builder()
        .system_with_handle(sleep_system(50), 0)
        .system_with_handle_and_deps(sleep_system(50), 1, vec![0])
        .system_with_handle_and_deps(sleep_system(50), 2, vec![0, 1])
        .system_with_deps(sleep_system(50), vec![0, 1, 2])
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, ()));
    assert!(time.elapsed() > Duration::from_millis(200));
}

#[test]
fn resources_incompatible_mutable_immutable() {
    let world = World::new();
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(|_, _: &A, _: ()| {
            sleep_millis(100);
        })
        .system(|_, a: &mut A, _: ()| {
            a.0 += 1;
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    assert!(time.elapsed() > Duration::from_millis(200));
    assert_eq!(a.0, 1);
}

#[test]
fn resources_incompatible_mutable_mutable() {
    let world = World::new();
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(|_, a: &mut A, _: ()| {
            a.0 += 1;
            sleep_millis(100);
        })
        .system(|_, a: &mut A, _: ()| {
            a.0 += 1;
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    assert!(time.elapsed() > Duration::from_millis(200));
    assert_eq!(a.0, 2);
}

#[test]
fn resources_disjoint() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, (a, c): (&mut A, &C), _: ()| {
            a.0 += c.0;
            sleep_millis(100);
        })
        .system(|_, (b, c): (&mut B, &C), _: ()| {
            b.0 += c.0;
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, (&mut a, &mut b, &mut c)));
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    assert_eq!(a.0, 2);
    assert_eq!(b.0, 3);
}

#[test]
fn queries_incompatible_mutable_immutable() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (B(0),)));
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, _: &A, q: QueryMarker<&B>| {
            for (_, _) in context.query(q).iter() {}
            sleep_millis(100);
        })
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    assert!(time.elapsed() > Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

#[test]
fn queries_incompatible_mutable_mutable() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (B(0),)));
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    assert!(time.elapsed() > Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 2);
    }
}

#[test]
fn queries_disjoint_by_components() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (B(0), C(0))));
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .system(|context, a: &A, q: QueryMarker<&mut C>| {
            for (_, c) in context.query(q).iter() {
                c.0 += a.0;
            }
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, (b, c)) in world.query::<(&B, &C)>().iter() {
        assert_eq!(b.0, 1);
        assert_eq!(c.0, 1);
    }
}

#[test]
fn queries_disjoint_by_archetypes() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (A(0), B(0))));
    world.spawn_batch((0..10).map(|_| (B(0), C(0))));
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<(&A, &mut B)>| {
            for (_, (_, b)) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .system(|context, a: &A, q: QueryMarker<(&mut B, &C)>| {
            for (_, (b, _)) in context.query(q).iter() {
                b.0 += a.0;
            }
            sleep_millis(100);
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

#[test]
fn batching() {
    let mut world = World::new();
    world.spawn_batch((0..20).map(|_| (B(0),)));
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            yaks::batch(&mut context.query(q), 4, |_, b| {
                b.0 += a.0;
                sleep_millis(10);
            });
        })
        .build();
    let time = Instant::now();
    execute(|| executor.run(&world, &mut a));
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

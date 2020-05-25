use hecs::World;
use std::{
    thread,
    time::{Duration, Instant},
};
use yaks::{Executor, QueryMarker};

struct A(usize);

struct B(usize);

struct C(usize);

#[cfg(feature = "parallel")]
fn thread_pool() -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new().build().unwrap()
}

#[cfg(not(feature = "parallel"))]
fn thread_pool() -> () {}

#[test]
fn dependencies_single() {
    let world = World::new();
    let mut executor = Executor::<()>::builder()
        .system_with_handle(
            |_, _: (), _: ()| {
                thread::sleep(Duration::from_millis(100));
            },
            0,
        )
        .system_with_handle_and_deps(
            |_, _: (), _: ()| {
                thread::sleep(Duration::from_millis(100));
            },
            1,
            vec![0],
        )
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, ());
    assert!(time.elapsed() > Duration::from_millis(200));
}

#[test]
fn resources_incompatible_mutable_immutable() {
    let world = World::new();
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(|_, _: &A, _: ()| {
            thread::sleep(Duration::from_millis(100));
        })
        .system(|_, a: &mut A, _: ()| {
            a.0 += 1;
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
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
            thread::sleep(Duration::from_millis(100));
        })
        .system(|_, a: &mut A, _: ()| {
            a.0 += 1;
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
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
            thread::sleep(Duration::from_millis(100));
        })
        .system(|_, (b, c): (&mut B, &C), _: ()| {
            b.0 += c.0;
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, (&mut a, &mut b, &mut c));
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
    world.spawn_batch((0..10).map(|_| (B(0),))).last();
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, _: &A, q: QueryMarker<&B>| {
            for (_, _) in context.query(q).iter() {}
            thread::sleep(Duration::from_millis(100));
        })
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
    assert!(time.elapsed() > Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

#[test]
fn queries_incompatible_mutable_mutable() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (B(0),))).last();
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
    assert!(time.elapsed() > Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 2);
    }
}

#[test]
fn queries_disjoint_by_components() {
    let mut world = World::new();
    world.spawn_batch((0..10).map(|_| (B(0), C(0)))).last();
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<&mut B>| {
            for (_, b) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .system(|context, a: &A, q: QueryMarker<&mut C>| {
            for (_, c) in context.query(q).iter() {
                c.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
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
    world.spawn_batch((0..10).map(|_| (A(0), B(0)))).last();
    world.spawn_batch((0..10).map(|_| (B(0), C(0)))).last();
    let mut a = A(1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &A, q: QueryMarker<(&A, &mut B)>| {
            for (_, (_, b)) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .system(|context, a: &A, q: QueryMarker<(&mut B, &C)>| {
            for (_, (b, _)) in context.query(q).iter() {
                b.0 += a.0;
            }
            thread::sleep(Duration::from_millis(100));
        })
        .build();
    let time = Instant::now();
    executor.run_on_thread_pool(&thread_pool(), &world, &mut a);
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

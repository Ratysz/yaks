use hecs::World;
use yaks::{Executor, Mut, Query, Ref};

struct A(usize);

struct B(usize);

struct C(usize);

#[test]
fn systems_single() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|a: &mut A, b: &B, c: &C| {
            a.0 = b.0 + c.0;
        })
        .build();
    executor.run(&world, (&mut a, &b, &c));
    assert_eq!(a.0, 3);
}

#[test]
fn systems_two() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|a: &mut A, b: &B| {
            a.0 += b.0;
        })
        .system(|a: &mut A, c: &C| {
            a.0 += c.0;
        })
        .build();
    executor.run(&world, (&mut a, &b, &c));
    assert_eq!(a.0, 3);
}

#[test]
fn resources_decoding_single() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|a: &mut A| {
            a.0 = 1;
        })
        .build();
    executor.run(&world, (&mut a, &b, &c));
    assert_eq!(a.0, 1);
}

#[test]
fn resources_wrap_single() {
    let world = World::new();
    let mut a = A(0);
    let mut executor = Executor::<Mut<A>>::builder()
        .system(|a: &mut A| {
            a.0 = 1;
        })
        .build();
    executor.run(&world, &mut a);
    assert_eq!(a.0, 1);
    let mut executor = Executor::<(Mut<A>,)>::builder()
        .system(|a: &mut A| {
            a.0 = 2;
        })
        .build();
    executor.run(&world, (&mut a,));
    assert_eq!(a.0, 2);
    let mut executor = Executor::<Mut<A>>::builder()
        .system(|a: &mut A| {
            a.0 = 3;
        })
        .build();
    executor.run(&world, (&mut a,));
    assert_eq!(a.0, 3);
    let mut executor = Executor::<(Mut<A>,)>::builder()
        .system(|a: &mut A| {
            a.0 = 4;
        })
        .build();
    executor.run(&world, &mut a);
    assert_eq!(a.0, 4);
}

#[test]
fn queries_decoding_single() {
    let mut world = World::new();
    world.spawn((B(1),));
    world.spawn((B(2),));
    let mut a = A(0);
    let mut executor = Executor::<Mut<A>>::builder()
        .system(|a: &mut A, query: Query<&B>| {
            for (_, b) in query.query().iter() {
                a.0 += b.0;
            }
        })
        .build();
    executor.run(&world, &mut a);
    assert_eq!(a.0, 3);
}

#[test]
#[allow(clippy::type_complexity)]
fn queries_decoding_four() {
    let mut world = World::new();
    world.spawn((B(1),));
    world.spawn((B(1),));
    world.spawn((A(0), B(1)));
    world.spawn((A(0),));
    world.spawn((C(2),));
    world.spawn((B(1), C(2)));
    let mut a = A(0);
    let mut executor = Executor::<Mut<A>>::builder()
        .system(
            |a: &mut A, q0: Query<&B>, q1: Query<(&A, &B)>, q2: Query<&C>, q3: Query<(&B, &C)>| {
                for (_, b) in q0.query().iter() {
                    a.0 += b.0;
                }
                assert_eq!(a.0, 4);
                a.0 = 0;
                for (_, (_, b)) in q1.query().iter() {
                    a.0 += b.0;
                }
                assert_eq!(a.0, 1);
                a.0 = 0;
                for (_, c) in q2.query().iter() {
                    a.0 += c.0;
                }
                assert_eq!(a.0, 4);
                a.0 = 0;
                for (_, (b, c)) in q3.query().iter() {
                    a.0 += b.0 + c.0;
                }
                assert_eq!(a.0, 3);
            },
        )
        .build();
    executor.run(&world, &mut a);
}

#[test]
#[should_panic(expected = "cannot borrow executor::A immutably: already borrowed mutably")]
fn invalid_resources_mutable_immutable() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|_: &mut A, _: &A| {})
        .build();
    executor.run(&world, (&mut a, &b, &c));
}

#[test]
#[should_panic(expected = "cannot borrow executor::A mutably: already borrowed")]
fn invalid_resources_immutable_mutable() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|_: &A, _: &mut A| {})
        .build();
    executor.run(&world, (&mut a, &b, &c));
}

#[test]
#[should_panic(expected = "cannot borrow executor::A mutably: already borrowed")]
fn invalid_resources_mutable_mutable() {
    let world = World::new();
    let mut a = A(0);
    let b = B(1);
    let c = C(2);
    let mut executor = Executor::<(Mut<A>, Ref<B>, Ref<C>)>::builder()
        .system(|_: &mut A, _: &mut A| {})
        .build();
    executor.run(&world, (&mut a, &b, &c));
}

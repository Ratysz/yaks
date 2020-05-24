use hecs::World;
use yaks::{Executor, QueryMarker};

struct A(usize);

struct B(usize);

struct C(usize);

#[test]
fn one_system() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, (a, b, c): (&mut A, &B, &C), _: ()| {
            a.0 = b.0 + c.0;
        })
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
    assert_eq!(a.0, 3);
}

#[test]
fn two_systems() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, (a, b): (&mut A, &B), _: ()| {
            a.0 += b.0;
        })
        .system(|_, (a, c): (&mut A, &C), _: ()| {
            a.0 += c.0;
        })
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
    assert_eq!(a.0, 3);
}

#[test]
fn single_resource_decoding() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, a: &mut A, _: ()| {
            a.0 = 1;
        })
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
    assert_eq!(a.0, 1);
}

#[test]
fn single_resource_wrapping() {
    let world = World::new();
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(|_, a: &mut A, _: ()| {
            a.0 = 1;
        })
        .build();
    executor.run(&world, (&mut a,));
    assert_eq!(a.0, 1);
    let mut executor = Executor::<(A,)>::builder()
        .system(|_, a: &mut A, _: ()| {
            a.0 = 2;
        })
        .build();
    executor.run(&world, &mut a);
    assert_eq!(a.0, 2);
}

#[test]
fn single_query_decoding() {
    let mut world = World::new();
    world.spawn((B(1),));
    world.spawn((B(2),));
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(|context, a: &mut A, query: QueryMarker<&B>| {
            for (_, b) in context.query(query).iter() {
                a.0 += b.0;
            }
        })
        .build();
    executor.run(&world, &mut a);
    assert_eq!(a.0, 3);
}

#[test]
fn four_queries_decoding() {
    let mut world = World::new();
    world.spawn((B(1),));
    world.spawn((B(1),));
    world.spawn((A(0), B(1)));
    world.spawn((A(0),));
    world.spawn((C(2),));
    world.spawn((B(1), C(2)));
    let mut a = A(0);
    let mut executor = Executor::<(A,)>::builder()
        .system(
            |context,
             a: &mut A,
             (q0, q1, q2, q3): (
                QueryMarker<&B>,
                QueryMarker<(&A, &B)>,
                QueryMarker<&C>,
                QueryMarker<(&B, &C)>,
            )| {
                for (_, b) in context.query(q0).iter() {
                    a.0 += b.0;
                }
                assert_eq!(a.0, 4);
                a.0 = 0;
                for (_, (_, b)) in context.query(q1).iter() {
                    a.0 += b.0;
                }
                assert_eq!(a.0, 1);
                a.0 = 0;
                for (_, c) in context.query(q2).iter() {
                    a.0 += c.0;
                }
                assert_eq!(a.0, 4);
                a.0 = 0;
                for (_, (b, c)) in context.query(q3).iter() {
                    a.0 += b.0 + c.0;
                }
                assert_eq!(a.0, 3);
            },
        )
        .build();
    executor.run(&world, &mut a);
}

#[cfg(not(feature = "parallel"))]
#[test]
#[should_panic(expected = "cannot borrow tests::A immutably: already borrowed mutably")]
fn invalid_resources_mutable_immutable() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, _: (&mut A, &A), _: ()| {})
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
}

#[cfg(not(feature = "parallel"))]
#[test]
#[should_panic(expected = "cannot borrow tests::A mutably: already borrowed")]
fn invalid_resources_immutable_mutable() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, _: (&A, &mut A), _: ()| {})
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
}

#[cfg(not(feature = "parallel"))]
#[test]
#[should_panic(expected = "cannot borrow tests::A mutably: already borrowed")]
fn invalid_resources_mutable_mutable() {
    let world = World::new();
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);
    let mut executor = Executor::<(A, B, C)>::builder()
        .system(|_, _: (&mut A, &mut A), _: ()| {})
        .build();
    executor.run(&world, (&mut a, &mut b, &mut c));
}

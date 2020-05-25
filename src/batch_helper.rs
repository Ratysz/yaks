use hecs::{Entity, Fetch, Query, QueryBorrow};

#[cfg_attr(not(feature = "parallel"), allow(unused_variables))]
pub fn batch<'query, 'world, Q, F>(
    query_borrow: &'query mut QueryBorrow<'world, Q>,
    batch_size: u32,
    for_each: F,
) where
    Q: Query + Send + Sync + 'query,
    F: Fn(Entity, <<Q as Query>::Fetch as Fetch<'query>>::Item) + Send + Sync,
{
    #[cfg(feature = "parallel")]
    {
        let iterator = query_borrow.iter_batched(batch_size);
        rayon::scope(|scope| {
            iterator.for_each(|batch| {
                scope.spawn(|_| {
                    batch.for_each(|(entity, components)| for_each(entity, components));
                });
            });
        });
    }
    #[cfg(not(feature = "parallel"))]
    {
        query_borrow
            .iter()
            .for_each(|(entity, components)| for_each(entity, components));
    }
}

#[cfg(feature = "parallel")]
#[test]
fn thread_pool_installation() {
    use hecs::World;
    use std::{
        thread,
        time::{Duration, Instant},
    };
    struct A(usize);
    struct B(usize);

    let mut world = World::new();
    world.spawn_batch((0..20).map(|_| (B(0),))).last();
    let a = A(1);
    let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
    let time = Instant::now();
    thread_pool.install(|| {
        batch(&mut world.query::<&mut B>(), 4, |_, b| {
            b.0 += a.0;
            thread::sleep(Duration::from_millis(10));
        });
    });
    #[cfg(not(feature = "parallel"))]
    assert!(time.elapsed() > Duration::from_millis(200));
    #[cfg(feature = "parallel")]
    assert!(time.elapsed() < Duration::from_millis(200));
    for (_, b) in world.query::<&B>().iter() {
        assert_eq!(b.0, 1);
    }
}

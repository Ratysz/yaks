use hecs::{Entity, Fetch, Query, QueryBorrow};

#[cfg_attr(not(feature = "parallel"), allow(unused_variables))]
/// Distributes over a `rayon` thread pool the work of applying a function to items in a query.
/// See [`hecs::QueryBorrow::batched_ter()`](../hecs/struct.QueryBorrow.html#method.iter_batched).
///
/// If the default `parallel` feature is disabled the functionality is identical
/// to `query_borrow.iter().for_each(for_each)`.
///
/// Calling `batch()` standalone will use the global `rayon` thread pool:
/// ```rust
/// # struct Pos;
/// # struct Vel;
/// # impl std::ops::AddAssign<&Vel> for Pos {
/// #     fn add_assign(&mut self, _: &Vel) {}
/// # }
/// # let world = hecs::World::new();
/// # let num_entities = 64;
/// yaks::batch(
///     &mut world.query::<(&mut Pos, &Vel)>(),
///     num_entities / 16,
///     |_entity, (pos, vel)| {
///         *pos += vel;
///     },
/// );
/// ```
/// Alternatively, a specific thread pool can be used via `rayon::ThreadPool::install()`:
/// ```rust
/// # struct Pos;
/// # struct Vel;
/// # impl std::ops::AddAssign<&Vel> for Pos {
/// #     fn add_assign(&mut self, _: &Vel) {}
/// # }
/// # let world = hecs::World::new();
/// # let num_entities = 64;
/// # #[cfg(feature = "parallel")]
/// # let thread_pool =
/// # {
/// #     rayon::ThreadPoolBuilder::new().build().unwrap()
/// # };
/// # #[cfg(not(feature = "parallel"))]
/// # let thread_pool =
/// # {
/// #     struct DummyPool;
/// #     impl DummyPool {
/// #         fn install(&self, closure: impl Fn()) {
/// #             closure();
/// #         }
/// #     }
/// #     DummyPool
/// # };
/// thread_pool.install(|| {
///     yaks::batch(
///         &mut world.query::<(&mut Pos, &Vel)>(),
///         num_entities / 16,
///         |_entity, (pos, vel)| {
///             *pos += vel;
///         },
///     )
/// });
/// ```
/// `batch()` can be called in systems, where it will use whichever thread pool is used by
/// the system or the executor it's in:
/// ```rust
/// # use yaks::{QueryMarker, Executor};
/// # struct Pos;
/// # struct Vel;
/// # impl std::ops::AddAssign<&Vel> for Pos {
/// #     fn add_assign(&mut self, _: &Vel) {}
/// # }
/// # let world = hecs::World::new();
/// # let mut num_entities = 64;
/// # #[cfg(feature = "parallel")]
/// # let thread_pool =
/// # {
/// #     rayon::ThreadPoolBuilder::new().build().unwrap()
/// # };
/// # #[cfg(not(feature = "parallel"))]
/// # let thread_pool =
/// # {
/// #     struct DummyPool;
/// #     impl DummyPool {
/// #         fn install(&self, mut closure: impl FnMut()) {
/// #             closure();
/// #         }
/// #     }
/// #     DummyPool
/// # };
/// let mut executor = Executor::<(u32, )>::builder()
///     .system(|context, num_entities: &u32, query: QueryMarker<(&mut Pos, &Vel)>| {
///         yaks::batch(
///             &mut context.query(query),
///             num_entities / 16,
///             |_entity, (pos, vel)| {
///                 *pos += vel;
///             },
///         )
///     })
///     .build();
///
/// executor.run(&world, &mut num_entities);
///
/// thread_pool.install(|| {
///     executor.run(&world, &mut num_entities);
/// });
/// ```
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
        use rayon::prelude::{ParallelBridge, ParallelIterator};
        query_borrow
            .iter_batched(batch_size)
            .par_bridge()
            .for_each(|batch| batch.for_each(|(entity, components)| for_each(entity, components)));
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
    world.spawn_batch((0..20).map(|_| (B(0),)));
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

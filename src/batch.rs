use hecs::{Entity, Fetch, Query, QueryBorrow};

#[cfg_attr(not(feature = "parallel"), allow(unused_variables))]
/// Distributes over a `rayon` thread pool the work of applying a function to items in a query.
/// See [`hecs::QueryBorrow::iter_batched()`](../hecs/struct.QueryBorrow.html#method.iter_batched).
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
/// Alternatively, a specific thread pool can be used via
/// [`rayon::ThreadPool::install()`](../rayon/struct.ThreadPool.html#method.install):
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
/// #         fn install(&self, mut closure: impl FnMut()) {
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
/// #     rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap()
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

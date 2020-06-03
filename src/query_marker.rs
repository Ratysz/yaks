use hecs::Query;
use std::marker::PhantomData;

/// A zero-sized `Copy` type used to describe queries of a system, and prepare them
/// via methods of [`SystemContext`](struct.SystemContext.html).
///
/// Instantiating these directly is only useful when calling systems as plain functions,
/// and can be done either by `QueryMarker::new()`, `QueryMarker::default()`, or
/// `Default::default()`, the latter of which can also instantiate tuples of markers (up to 10):
/// ```rust
/// # use yaks::{SystemContext, QueryMarker};
/// # let world = hecs::World::new();
/// # let world = &world;
/// # let mut average = 0f32;
/// fn single_query(context: SystemContext, average: &mut f32, query: QueryMarker<&f32>) {
///     *average = 0f32;
///     let mut entities = 0;
///     for (_entity, value) in context.query(query).iter() {
///         entities += 1;
///         *average += *value;
///     }
///     *average /= entities as f32;
/// }
///
/// single_query(world.into(), &mut average, QueryMarker::new());
/// single_query(world.into(), &mut average, QueryMarker::default());
/// single_query(world.into(), &mut average, Default::default());
///
/// fn two_queries(
///     context: SystemContext,
///     average: &mut f32,
///     (floats, ints): (QueryMarker<&f32>, QueryMarker<&i32>),
/// ) {
///     *average = 0f32;
///     let mut entities = 0;
///     for (_entity, value) in context.query(floats).iter() {
///         entities += 1;
///         *average += *value;
///     }
///     for (_entity, value) in context.query(ints).iter() {
///         entities += 1;
///         *average += *value as f32;
///     }
///     *average /= entities as f32;
/// }
///
/// two_queries(world.into(), &mut average, Default::default());
/// ```
/// # Instantiating markers inside a system is improper!
/// While it's possible to instantiate a marker within a system and use it to prepare a query,
/// doing so does not inform the executor the system may be in of said query,
/// and may lead to a panic.
pub struct QueryMarker<Q0>(PhantomData<Q0>)
where
    Q0: Query;

impl<Q0> QueryMarker<Q0>
where
    Q0: Query,
{
    /// Equivalent to `QueryMarker::default()`. See documentation for `QueryMarker` itself.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Q0> Clone for QueryMarker<Q0>
where
    Q0: Query,
{
    fn clone(&self) -> Self {
        QueryMarker::new()
    }
}

impl<Q0> Copy for QueryMarker<Q0> where Q0: Query {}

impl<Q0> Default for QueryMarker<Q0>
where
    Q0: Query,
{
    fn default() -> Self {
        Self::new()
    }
}

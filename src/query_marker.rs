use hecs::Query;
use std::marker::PhantomData;

/// A zero-sized `Copy` type used to describe queries of a system, and prepare them
/// via methods of [`SystemContext`](struct.SystemContext.html).
///
/// It cannot be instantiated directly. See [`System`](trait.System.html) for instructions
/// on how to call systems outside of an executor, as plain functions.
pub struct QueryMarker<Q0>(PhantomData<Q0>)
where
    Q0: Query;

impl<Q0> QueryMarker<Q0>
where
    Q0: Query,
{
    pub(crate) fn new() -> Self {
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

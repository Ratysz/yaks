use std::marker::PhantomData;

/// A zero-sized `Copy` type used to describe queries of a system, and prepare them
/// via methods of [`SystemContext`](struct.SystemContext.html).
///
/// It cannot be instantiated directly. See [`System`](trait.System.html) for instructions
/// on how to call systems outside of an executor, as plain functions.
pub struct Query<'a, Q0>
where
    Q0: hecs::Query,
{
    phantom_data: PhantomData<Q0>,
    world: &'a hecs::World,
}

impl<'a, Q0> Query<'a, Q0>
where
    Q0: hecs::Query,
{
    pub(crate) fn new(world: &'a hecs::World) -> Self {
        Self {
            phantom_data: PhantomData,
            world,
        }
    }

    /// Prepares the query; see [`hecs::World::query()`](../hecs/struct.World.html#method.query).
    ///
    /// # Example
    /// ```rust
    /// # use yaks::{Query};
    /// # struct Pos;
    /// # #[derive(Clone, Copy)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(
    ///     _resources: (),
    ///     pos_vel: Query<(&mut Pos, &Vel)>
    /// ) {
    ///     for (_entity, (pos, vel)) in pos_vel.query().iter() {
    ///         *pos += *vel;
    ///     }
    /// };
    /// ```
    pub fn query(&self) -> hecs::QueryBorrow<Q0> {
        self.world.query()
    }

    /// Prepares the query against a single entity;
    /// see [`hecs::World::query_one()`](../hecs/struct.World.html#method.query_one).
    ///
    /// # Example
    /// ```rust
    /// # use yaks::{Query};
    /// # #[derive(Default)]
    /// # struct Pos;
    /// # #[derive(Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(
    ///     _resources: (),
    ///     pos_vel: Query<(&mut Pos, &Vel)>
    /// ) {
    ///     let mut max_velocity = Vel::default();
    ///     let mut max_velocity_entity = None;
    ///     for (entity, (pos, vel)) in pos_vel.query().iter() {
    ///         *pos += *vel;
    ///         if *vel > max_velocity {
    ///             max_velocity = *vel;
    ///             max_velocity_entity = Some(entity);
    ///         }
    ///     }
    ///     if let Some(entity) = max_velocity_entity {
    ///         let mut query_one = pos_vel
    ///             .query_one(entity)
    ///             .expect("no such entity");
    ///         let (pos, _vel) = query_one
    ///             .get()
    ///             .expect("some components are missing");
    ///         *pos = Pos::default();
    ///     }
    /// };
    /// ```
    pub fn query_one(
        &self,
        entity: hecs::Entity,
    ) -> Result<hecs::QueryOne<Q0>, hecs::NoSuchEntity> {
        self.world.query_one(entity)
    }
}

impl<Q0> Clone for Query<'_, Q0>
where
    Q0: hecs::Query,
{
    fn clone(&self) -> Self {
        Query {
            phantom_data: PhantomData,
            world: self.world,
        }
    }
}

impl<Q0> Copy for Query<'_, Q0> where Q0: hecs::Query {}

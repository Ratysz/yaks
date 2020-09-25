use std::marker::PhantomData;

/// A thin wrapper over [`&hecs::World`](../hecs/struct.World.html),
/// used to describe and prepare queries of a system.
///
/// It can be copied, but cannot be instantiated directly.
/// Use [`Run`](trait.Run.html) to call systems as plain functions.
pub struct Query<'a, Q>
where
    Q: hecs::Query,
{
    phantom_data: PhantomData<Q>,
    world: &'a hecs::World,
}

impl<Q> Clone for Query<'_, Q>
where
    Q: hecs::Query,
{
    fn clone(&self) -> Self {
        Query {
            phantom_data: PhantomData,
            world: self.world,
        }
    }
}

impl<Q> Copy for Query<'_, Q> where Q: hecs::Query {}

impl<'a, Q> Query<'a, Q>
where
    Q: hecs::Query,
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
    /// # use yaks::Query;
    /// # struct Pos;
    /// # #[derive(Clone, Copy)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(pos_vel: Query<(&mut Pos, &Vel)>) {
    ///     for (_entity, (pos, vel)) in pos_vel.query().iter() {
    ///         *pos += *vel;
    ///     }
    /// };
    /// ```
    pub fn query(&self) -> hecs::QueryBorrow<Q> {
        self.world.query()
    }

    /// Prepares the query against a single entity;
    /// see [`hecs::World::query_one()`](../hecs/struct.World.html#method.query_one).
    ///
    /// # Example
    /// ```rust
    /// # use yaks::Query;
    /// # #[derive(Default)]
    /// # struct Pos;
    /// # #[derive(Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(pos_vel: Query<(&mut Pos, &Vel)>) {
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
    pub fn query_one(&self, entity: hecs::Entity) -> Result<hecs::QueryOne<Q>, hecs::NoSuchEntity> {
        self.world.query_one(entity)
    }
}

use hecs::{
    Archetype, ArchetypesGeneration, Entity, NoSuchEntity, Query, QueryBorrow, QueryOne, World,
};

use crate::{QueryMarker, SystemId};

/// Thin wrapper over [`hecs::World`](../hecs/struct.World.html), can prepare queries using a
/// [`QueryMarker`](struct.QueryMarker.html).
///
/// It cannot be instantiated directly. See [`System`](trait.System.html) for instructions
/// on how to call systems outside of an executor, as plain functions.
pub struct SystemContext<'scope> {
    pub(crate) system_id: Option<SystemId>,
    pub(crate) world: &'scope World,
}

impl<'scope> SystemContext<'scope> {
    /// Returns a debug-printable `SystemId` if the system is ran in an
    /// [`Executor`](struct.Executor.html), with printed number reflecting
    /// the order of insertion into the [`ExecutorBuilder`](struct.ExecutorBuilder.html).
    pub fn id(&self) -> Option<SystemId> {
        self.system_id
    }

    /// Prepares a query using the given [`QueryMarker`](struct.QueryMarker.html);
    /// see [`hecs::World::query()`](../hecs/struct.World.html#method.query).
    ///
    /// # Example
    /// ```rust
    /// # use yaks::{SystemContext, QueryMarker};
    /// # struct Pos;
    /// # #[derive(Clone, Copy)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(
    ///     context: SystemContext,
    ///     _resources: (),
    ///     query: QueryMarker<(&mut Pos, &Vel)>
    /// ) {
    ///     for (_entity, (pos, vel)) in context.query(query).iter() {
    ///         *pos += *vel;
    ///     }
    /// };
    /// ```
    pub fn query<Q>(&self, _: QueryMarker<Q>) -> QueryBorrow<'_, Q>
    where
        Q: Query + Send + Sync,
    {
        self.world.query()
    }

    /// Prepares a query against a single entity using the given
    /// [`QueryMarker`](struct.QueryMarker.html);
    /// see [`hecs::World::query_one()`](../hecs/struct.World.html#method.query_one).
    ///
    /// # Example
    /// ```rust
    /// # use yaks::{SystemContext, QueryMarker};
    /// # #[derive(Default)]
    /// # struct Pos;
    /// # #[derive(Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
    /// # struct Vel;
    /// # impl std::ops::AddAssign<Vel> for Pos {
    /// #     fn add_assign(&mut self, _: Vel) {}
    /// # }
    /// # let world = hecs::World::new();
    /// fn some_system(
    ///     context: SystemContext,
    ///     _resources: (),
    ///     query: QueryMarker<(&mut Pos, &Vel)>
    /// ) {
    ///     let mut max_velocity = Vel::default();
    ///     let mut max_velocity_entity = None;
    ///     for (entity, (pos, vel)) in context.query(query).iter() {
    ///         *pos += *vel;
    ///         if *vel > max_velocity {
    ///             max_velocity = *vel;
    ///             max_velocity_entity = Some(entity);
    ///         }
    ///     }
    ///     if let Some(entity) = max_velocity_entity {
    ///         let mut query_one = context
    ///             .query_one(query, entity)
    ///             .expect("no such entity");
    ///         let (pos, _vel) = query_one
    ///             .get()
    ///             .expect("some components are missing");
    ///         *pos = Pos::default();
    ///     }
    /// };
    /// ```
    pub fn query_one<Q>(
        &self,
        _: QueryMarker<Q>,
        entity: Entity,
    ) -> Result<QueryOne<'_, Q>, NoSuchEntity>
    where
        Q: Query + Send + Sync,
    {
        self.world.query_one(entity)
    }

    /// See [`hecs::World::reserve_entity()`](../hecs/struct.World.html#method.reserve_entity).
    pub fn reserve_entity(&self) -> Entity {
        self.world.reserve_entity()
    }

    /// See [`hecs::World::contains()`](../hecs/struct.World.html#method.contains).
    pub fn contains(&self, entity: Entity) -> bool {
        self.world.contains(entity)
    }

    /// See [`hecs::World::archetypes()`](../hecs/struct.World.html#method.archetypes).
    pub fn archetypes(&self) -> impl ExactSizeIterator<Item = &Archetype> + '_ {
        self.world.archetypes()
    }

    /// See [`hecs::World::archetypes_generation()`][ag].
    ///
    /// [ag]: ../hecs/struct.World.html#method.archetypes_generation
    pub fn archetypes_generation(&self) -> ArchetypesGeneration {
        self.world.archetypes_generation()
    }
}

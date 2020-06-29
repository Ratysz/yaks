use hecs::World;
use std::collections::HashMap;

use crate::{
    DummyHandle, ExecutorBuilder, ResourceTuple, ResourceWrap, SystemContext, WrappedResources,
};

#[cfg(feature = "parallel")]
use crate::{ExecutorParallel, TypeSet};

#[cfg(not(feature = "parallel"))]
use crate::SystemId;

pub type SystemClosure<'closure, Cells> =
    dyn FnMut(SystemContext, &WrappedResources<Cells>) + Send + Sync + 'closure;

/// A sealed container for systems that may be executed in parallel.
///
/// Systems can be any closure or function that return nothing and have these 3 arguments:
/// - [`SystemContext`](struct.SystemContext.html),
/// - any tuple (up to 16) or a single one of "resources": references or mutable references
/// to `Send + Sync` values not contained in a [`hecs::World`](../hecs/struct.World.html)
/// that the system will be accessing,
/// - any tuple (up to 16) or a single one of [`QueryMarker`](struct.QueryMarker.html) that
/// represent the queries the system will be making.
///
/// Additionally, closures may mutably borrow from their environment for the lifetime `'closures`
/// of the executor, but must be `Send + Sync`. If none of the systems make any borrows from the
/// environment, said lifetime can simply be `'static`.
///
/// The generic parameter `Resources` of the executor must be a superset tuple of all resource set
/// tuples of the contained systems. Any type in `Resources` must appear no more than once,
/// however, any number of systems in the executor may have either an immutable or a mutable
/// reference of said type in their signature. For example: if any number of systems require
/// a `&f32` or a `&mut f32`, `Resources` must contain `f32`.
///
/// It's possible to define an order of execution of the systems by building up a dependency
/// graph when building the executor, see [`ExecutorBuilder::system_with_handle()`][swh].
///
/// [swh]: struct.ExecutorBuilder.html#method.system_with_handle
///
/// Executors are relatively costly to instantiate, and should be cached whenever possible.
///
/// Executors are not intended to house any and all behavior of the program, they work best
/// when treated as a sort of [`yaks::batch()`](fn.batch.html) for systems; e.g.,
/// make one only when the systems in it may actually benefit from being ran concurrently
/// and prefer several small executors over a single large one.
///
/// See [`::run()`](#method.run), crate examples, and documentation for other items in the library
/// for more details and specific demos.
pub struct Executor<'closures, Resources>
where
    Resources: ResourceTuple,
{
    #[cfg(feature = "parallel")]
    inner: ExecutorParallel<'closures, Resources>,
    #[cfg(not(feature = "parallel"))]
    inner: ExecutorSequential<'closures, Resources>,
}

impl<'closures, Resources> Executor<'closures, Resources>
where
    Resources: ResourceTuple,
{
    /// Creates a new [`ExecutorBuilder`](struct.ExecutorBuilder.html).
    pub fn builder() -> ExecutorBuilder<'closures, Resources> {
        ExecutorBuilder::<'closures, Resources, DummyHandle> {
            systems: HashMap::new(),
            handles: HashMap::with_capacity(0),
            #[cfg(feature = "parallel")]
            all_component_types: TypeSet::new(),
        }
    }

    pub(crate) fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        Self {
            #[cfg(feature = "parallel")]
            inner: ExecutorParallel::build(builder),
            #[cfg(not(feature = "parallel"))]
            inner: ExecutorSequential::build(builder),
        }
    }

    /// Forces the executor to forget stored [`hecs::ArchetypesGeneration`][1], see
    /// [`hecs::World::archetypes_generation()`][2].
    ///
    /// **Must** be called before using the executor with a different [`hecs::World`][3] than
    /// it was used with earlier - not doing so may cause a panic when a query makes it's borrows.
    /// In all other cases, calling this function is unnecessary and detrimental to performance.
    ///
    /// [1]: ../hecs/struct.ArchetypesGeneration.html
    /// [2]: ../hecs/struct.World.html#method.archetypes_generation
    /// [3]: ../hecs/struct.World.html
    ///
    /// # Example
    /// ```rust
    /// # let mut executor = yaks::Executor::<()>::builder().build();
    /// # let world_a = hecs::World::new();
    /// # let world_b = hecs::World::new();
    /// executor.run(&world_a, ());
    /// executor.run(&world_a, ());
    /// executor.force_archetype_recalculation();
    /// executor.run(&world_b, ());
    /// executor.run(&world_b, ());
    /// executor.force_archetype_recalculation();
    /// executor.run(&world_a, ());
    /// ```
    pub fn force_archetype_recalculation(&mut self) {
        self.inner.force_archetype_recalculation();
    }

    /// Executes all of the contained systems once, running as much of them at the same time
    /// as their resource use, queries, and dependencies allow.
    ///
    /// The exact order of execution is not guaranteed, except for systems with defined
    /// dependencies (see [`ExecutorBuilder::system_with_handle()`][swh]), or if the default
    /// `parallel` feature is disabled (in which case the systems will be executed in order
    /// of their insertion into the builder).
    ///
    /// [swh]: struct.ExecutorBuilder.html#method.system_with_handle
    ///
    /// The `resources` argument when calling this function must be a tuple of exclusive references
    /// to values of types specified by the generic parameter `Resources` of the executor:
    /// ```rust
    /// # use yaks::Executor;
    /// # let world = hecs::World::new();
    /// let mut executor = Executor::<(f32, u32)>::builder().build();
    /// let mut some_f32 = 0f32;
    /// let mut some_u32 = 0u32;
    /// executor.run(&world, (&mut some_f32, &mut some_u32));
    ///
    /// let mut executor = Executor::<(f32, )>::builder().build();
    /// executor.run(&world, (&mut some_f32, ));
    ///
    /// // Single resource type is also special-cased for convenience.
    /// let mut executor = Executor::<(f32, )>::builder().build();
    /// executor.run(&world, &mut some_f32);
    ///
    /// let mut executor = Executor::<()>::builder().build();
    /// executor.run(&world, ());
    /// ```
    /// In the future, exclusivity requirement might be relaxed for resources that aren't mutated
    /// by any of the systems, but doing that as of writing is unfeasible.
    ///
    /// This function can be called inside a
    /// [`rayon::ThreadPool::install()`](../rayon/struct.ThreadPool.html#method.install) block
    /// to use that thread pool instead of the global one:
    /// ```rust
    /// # use yaks::Executor;
    /// # let world = hecs::World::new();
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
    /// # let mut executor = Executor::<()>::builder().build();
    /// thread_pool.install(|| executor.run(&world, ()));
    /// ```
    /// Doing so will cause all [`yaks::batch()`](fn.batch.html) calls inside systems
    /// to also use said thread pool.
    ///
    /// # Panics
    /// This function will panic if:
    /// - a system within the executor has resource requirements that are incompatible with itself,
    /// e.g. `(&mut SomeResource, &SomeResource)`.
    ///
    /// Additionally, it *may* panic if:
    /// - a different [`hecs::World`](../hecs/struct.World.html) is supplied than in a previous
    /// call, without first calling
    /// [`::force_archetype_recalculation()`](#method.force_archetype_recalculation).
    pub fn run<ResourceTuple>(&mut self, world: &World, resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        self.inner.run(world, resources);
    }
}

#[cfg(not(feature = "parallel"))]
struct ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    borrows: Resources::Borrows,
    systems: Vec<(SystemId, Box<SystemClosure<'closures, Resources::Cells>>)>,
}

#[cfg(not(feature = "parallel"))]
impl<'closures, Resources> ExecutorSequential<'closures, Resources>
where
    Resources: ResourceTuple,
{
    fn build<Handle>(builder: ExecutorBuilder<'closures, Resources, Handle>) -> Self {
        let ExecutorBuilder { mut systems, .. } = builder;
        let mut systems: Vec<_> = systems
            .drain()
            .map(|(id, system)| (id, system.closure))
            .collect();
        systems.sort_by(|(a, _), (b, _)| a.cmp(b));
        ExecutorSequential {
            borrows: Resources::instantiate_borrows(),
            systems,
        }
    }

    fn force_archetype_recalculation(&mut self) {}

    fn run<ResourceTuple>(&mut self, world: &World, mut resources: ResourceTuple)
    where
        ResourceTuple: ResourceWrap<Cells = Resources::Cells, Borrows = Resources::Borrows> + Send,
        Resources::Borrows: Send,
        Resources::Cells: Send + Sync,
    {
        let wrapped = resources.wrap(&mut self.borrows);
        for (id, closure) in &mut self.systems {
            closure(
                SystemContext {
                    system_id: Some(*id),
                    world,
                },
                &wrapped,
            );
        }
    }
}

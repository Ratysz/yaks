//! An annotated non-trivial example. Runs with or without the `parallel` feature.

use hecs::World;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::{Duration, Instant};
use yaks::{Executor, QueryMarker, SystemContext};

// Each of the tests will be ran this many times.
const ITERATIONS: u32 = 100;

// A resource used to inform systems of how many entities they're working with,
// without explicitly counting them each time.
struct SpawnedEntities {
    no_acceleration: u32,
    with_acceleration: u32,
}

impl SpawnedEntities {
    // How many batches will a query be split into with `yaks::batch()`.
    const BATCH_TASKS: u32 = 32;

    // Determines how many entities will be in a batch.
    pub const fn batch_size_all(&self) -> u32 {
        (self.no_acceleration + self.with_acceleration) / Self::BATCH_TASKS
    }

    pub const fn batch_size_no_acceleration(&self) -> u32 {
        self.no_acceleration / Self::BATCH_TASKS
    }

    pub const fn batch_size_with_acceleration(&self) -> u32 {
        self.with_acceleration / Self::BATCH_TASKS
    }
}

// Example components and/or resources.
struct Position(f32, f32);
struct Velocity(f32, f32);
struct Acceleration(f32, f32);
struct Color(f32, f32, f32, f32);

// A system that simulates 2D kinematic motion.
#[allow(clippy::type_complexity)]
fn motion(
    // Thin wrapper over `&hecs::World`.
    context: SystemContext,
    // A resource this system requires. Can be a single one, or any tuple up to 16.
    spawned: &SpawnedEntities,
    // Queries this system will execute. Can be a single one, or any tuple up to 16.
    (no_acceleration, with_acceleration): (
        // `QueryMarker` is a zero-sized type that can be fed into methods of `SystemContext`.
        QueryMarker<hecs::Without<Acceleration, (&mut Position, &Velocity)>>,
        QueryMarker<(&mut Position, &mut Velocity, &Acceleration)>,
    ),
) {
    // A helper function that automatically spreads the batches across threads of a
    // `rayon::ThreadPool` - either the global one if called standalone, or a specific one
    // when used with a `rayon::ThreadPool::install()`.
    yaks::batch(
        &mut context.query(no_acceleration),
        spawned.batch_size_no_acceleration(),
        |_entity, (mut pos, vel)| {
            pos.0 += vel.0;
            pos.1 += vel.1;
        },
    );
    // If the default `parallel` feature is disabled this simply iterates in a single thread.
    yaks::batch(
        &mut context.query(with_acceleration),
        spawned.batch_size_with_acceleration(),
        |_entity, (mut pos, mut vel, acc)| {
            vel.0 += acc.0;
            vel.1 += acc.1;
            pos.0 += vel.0;
            pos.1 += vel.1;
        },
    );
}

// A system that tracks the highest velocity among all entities.
fn find_highest_velocity(
    context: SystemContext,
    highest: &mut Velocity,
    query: QueryMarker<&Velocity>,
) {
    // This cannot be batched as is because it needs mutable access to `highest`;
    // however, it's possible to work around that by using channels and/or `RwLock`.
    for (_entity, vel) in context.query(query).iter() {
        if vel.0 * vel.0 + vel.1 * vel.1 > highest.0 * highest.0 + highest.1 * highest.1 {
            highest.0 = vel.0;
            highest.1 = vel.1;
        }
    }
}

// A system that recolors entities based on their kinematic properties.
fn color(
    context: SystemContext,
    (spawned, rng): (&SpawnedEntities, &mut StdRng),
    query: QueryMarker<(&Position, &Velocity, &mut Color)>,
) {
    // Of course, it's possible to use resources mutably and still batch queries if
    // mutation happens outside batching.
    let blue = rng.gen_range(0.0, 1.0);
    yaks::batch(
        &mut context.query(query),
        spawned.batch_size_all(),
        |_entity, (pos, vel, mut col)| {
            col.0 = pos.0.abs() / 1000.0;
            col.1 = vel.1.abs() / 100.0;
            col.2 = blue;
        },
    );
}

// A system that tracks the average color of entities.
fn find_average_color(
    context: SystemContext,
    (average_color, spawned): (&mut Color, &SpawnedEntities),
    query: QueryMarker<&Color>,
) {
    *average_color = Color(0.0, 0.0, 0.0, 0.0);
    for (_entity, color) in context.query(query).iter() {
        average_color.0 += color.0;
        average_color.1 += color.1;
        average_color.2 += color.2;
        average_color.3 += color.3;
    }
    let entities = (spawned.no_acceleration + spawned.with_acceleration) as f32;
    average_color.0 /= entities;
    average_color.1 /= entities;
    average_color.2 /= entities;
    average_color.3 /= entities;
}

fn main() {
    // Trying to parse a passed argument, if any.
    let to_spawn: u32 = std::env::args()
        .nth(1)
        .ok_or(())
        .and_then(|arg| arg.parse::<u32>().map_err(|_| ()))
        .unwrap_or(100_000);
    // Initializing resources.
    let mut rng = StdRng::from_entropy();
    let mut world = World::new();
    let mut average_color = Color(0.0, 0.0, 0.0, 0.0);
    let mut highest_velocity = Velocity(0.0, 0.0);
    let mut spawned = SpawnedEntities {
        no_acceleration: 0,
        with_acceleration: 0,
    };

    // Spawning entities.
    world.spawn_batch((0..(to_spawn / 2)).map(|_| {
        spawned.no_acceleration += 1;
        (
            Position(rng.gen_range(-100.0, 100.0), rng.gen_range(-100.0, 100.0)),
            Velocity(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)),
            Color(0.0, 0.0, 0.0, 1.0),
        )
    }));
    assert!(spawned.no_acceleration >= SpawnedEntities::BATCH_TASKS);
    world.spawn_batch((0..(to_spawn / 2)).map(|_| {
        spawned.with_acceleration += 1;
        (
            Position(rng.gen_range(-100.0, 100.0), rng.gen_range(-100.0, 100.0)),
            Velocity(rng.gen_range(-10.0, 10.0), rng.gen_range(-10.0, 10.0)),
            Acceleration(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)),
            Color(0.0, 0.0, 0.0, 1.0),
        )
    }));
    assert!(spawned.with_acceleration >= SpawnedEntities::BATCH_TASKS);
    println!(
        "spawned {} entities",
        spawned.no_acceleration + spawned.with_acceleration
    );
    let world = &world;

    let mut iterations = 0u32;

    // The `Executor` is the main abstraction provided by `yaks`; it tries to execute
    // as much of it's systems at the same time as their borrows allow, while preserving
    // given order of execution, if any.
    // The generic parameter is the superset of resource sets of all of it's systems.
    let mut executor = Executor::<'_, (SpawnedEntities, StdRng, Color, Velocity)>::builder()
        // Handles and dependencies are optional,
        // can be of any type that is `Eq + Hash + Debug`,
        // and are discarded on `build()`.
        .system_with_handle(motion, "motion")
        // Systems can be defined by either a function or a closure
        // with a specific signature; see `ExecutorBuilder::system()` documentation.
        // The closures can also mutably borrow from their environment,
        // for the lifetime of the executor.
        // (Don't actually do this. Systems with no resources or queries have
        // no business being in an executor.)
        .system(|_context, _resources: (), _queries: ()| iterations += 1)
        // The builder will panic if given a system with a handle it already contains,
        // a list of dependencies with a system it doesn't contain yet,
        // or a system that depends on itself.
        .system_with_deps(find_highest_velocity, vec!["motion"])
        // Relative order of execution is only guaranteed for systems with explicit dependencies.
        // If the default `parallel` feature is disabled, systems are ran in order of insertion.
        .system_with_handle_and_deps(color, "color", vec!["motion"])
        .system_with_deps(find_average_color, vec!["color"])
        // Building is allocating, so executors should be cached whenever possible.
        .build();

    print!("running {} iterations of executor...  ", ITERATIONS);
    let mut elapsed = Duration::from_millis(0);
    for _ in 0..ITERATIONS {
        let time = Instant::now();
        // Running the executor requires a tuple of exclusive references to the resources
        // specified in it's generic parameter.
        // To use a specific `rayon` thread pool rather than the global one this function
        // should be called within `rayon::ThreadPool::install()` (which will have
        // any `yaks::batch()` calls in systems also use that thread pool).
        executor.run(
            world,
            (
                &mut spawned,
                &mut rng,
                &mut average_color,
                &mut highest_velocity,
            ),
        );
        elapsed += time.elapsed();
    }
    println!("average time: {:?}", elapsed / ITERATIONS);
    drop(executor); // Dropping the executor releases the borrow of `iterations`.
    assert_eq!(ITERATIONS, iterations);

    // The types appearing in system signatures have convenience constructors
    // to allow easily using systems as plain functions.
    print!("running {} iterations of functions... ", ITERATIONS);
    let mut elapsed = Duration::from_millis(0);
    for _ in 0..ITERATIONS {
        let time = Instant::now();
        // `SystemContext` can be constructed from `&hecs::World` or `&mut hecs::World`,
        // or via `SystemContext::new()`.
        motion(world.into(), &spawned, Default::default());
        // The zero-sized `QueryMarker` can be constructed by `QueryMarker::new()`; singles
        // or tuples of them can also be constructed via `Default::default()` (up to 10).
        find_highest_velocity(
            SystemContext::new(world),
            &mut highest_velocity,
            QueryMarker::new(),
        );
        color(world.into(), (&spawned, &mut rng), Default::default());
        find_average_color(
            world.into(),
            (&mut average_color, &spawned),
            Default::default(),
        );
        elapsed += time.elapsed();
    }
    println!("average time: {:?}", elapsed / ITERATIONS);

    // The `batch()` helper function can also be used outside of systems,
    // since the first argument is simply a `QueryBorrow`.
    // Again, calling this within `rayon::ThreadPool::install()` will use that thread pool.
    yaks::batch(
        &mut world.query::<&mut Color>(),
        spawned.batch_size_all(),
        |_entity, color| {
            color.3 = 0.5;
        },
    );
    find_average_color(
        world.into(),
        (&mut average_color, &spawned),
        Default::default(),
    );
    assert!((average_color.3 - 0.5).abs() < std::f32::EPSILON);
}

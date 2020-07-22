use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hecs::World;
use rand::{rngs::StdRng, Rng, SeedableRng};
use yaks::{Executor, QueryMarker, SystemContext};

struct SpawnedEntities {
    no_acceleration: u32,
    with_acceleration: u32,
}

impl SpawnedEntities {
    const BATCH_TASKS: u32 = 32;

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

struct Position(f32, f32);
struct Velocity(f32, f32);
struct Acceleration(f32, f32);
struct Color(f32, f32, f32, f32);

#[allow(clippy::type_complexity)]
fn motion(
    context: SystemContext,
    spawned: &SpawnedEntities,
    (no_acceleration, with_acceleration): (
        QueryMarker<hecs::Without<Acceleration, (&mut Position, &Velocity)>>,
        QueryMarker<(&mut Position, &mut Velocity, &Acceleration)>,
    ),
) {
    yaks::batch(
        &mut context.query(no_acceleration),
        spawned.batch_size_no_acceleration(),
        |_entity, (mut pos, vel)| {
            pos.0 += vel.0;
            pos.1 += vel.1;
        },
    );
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

fn find_highest_velocity(
    context: SystemContext,
    highest: &mut Velocity,
    query: QueryMarker<&Velocity>,
) {
    for (_entity, vel) in context.query(query).iter() {
        if vel.0 * vel.0 + vel.1 * vel.1 > highest.0 * highest.0 + highest.1 * highest.1 {
            highest.0 = vel.0;
            highest.1 = vel.1;
        }
    }
}

fn color(
    context: SystemContext,
    (spawned, rng): (&SpawnedEntities, &mut StdRng),
    query: QueryMarker<(&Position, &Velocity, &mut Color)>,
) {
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

fn convoluted(criterion: &mut Criterion) {
    let to_spawn: u32 = 100_000;
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
    world.spawn_batch((0..(to_spawn / 2)).map(|_| {
        spawned.with_acceleration += 1;
        (
            Position(rng.gen_range(-100.0, 100.0), rng.gen_range(-100.0, 100.0)),
            Velocity(rng.gen_range(-10.0, 10.0), rng.gen_range(-10.0, 10.0)),
            Acceleration(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)),
            Color(0.0, 0.0, 0.0, 1.0),
        )
    }));

    criterion.bench_function("Executor::run()", |bencher| {
        bencher.iter(|| {
            let mut executor: Executor<(SpawnedEntities, StdRng, Color, Velocity)> =
                Executor::builder()
                    .system_with_handle(motion, 0)
                    .system_with_deps(find_highest_velocity, vec![0])
                    .system_with_handle_and_deps(color, 1, vec![0])
                    .system_with_deps(find_average_color, vec![1])
                    .build();
            executor.run(
                &world,
                (
                    &mut spawned,
                    &mut rng,
                    &mut average_color,
                    &mut highest_velocity,
                ),
            )
        });
    });
}

criterion_group!(benches, convoluted);
criterion_main!(benches);

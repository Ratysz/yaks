//! Copy of the crate level documentation & readme example.

use hecs::{With, Without, World};
use yaks::{Executor, Mut, Query, Ref};

fn main() {
    let mut world = World::new();
    let mut entities = 0u32;
    world.spawn_batch((0..100u32).map(|index| {
        entities += 1;
        (index,)
    }));
    world.spawn_batch((0..100u32).map(|index| {
        entities += 1;
        (index, index as f32)
    }));
    let increment = 5usize;
    let mut average = 0f32;
    let mut executor = Executor::<(Ref<u32>, Ref<usize>, Mut<f32>)>::builder()
        .system_with_handle(
            |entities: &u32, average: &mut f32, floats: Query<&f32>| {
                *average = 0.0;
                for (_entity, float) in floats.query().iter() {
                    *average += *float;
                }
                *average /= *entities as f32;
            },
            "average",
        )
        .system_with_handle(
            |increment: &usize, unsigned: Query<&mut u32>| {
                for (_entity, unsigned) in unsigned.query().iter() {
                    *unsigned += *increment as u32
                }
            },
            "increment",
        )
        .system_with_deps(system_with_two_queries, vec!["increment", "average"])
        .build();
    executor.run(&world, (&entities, &increment, &mut average));
}

fn system_with_two_queries(
    entities: &u32,
    average: &f32,
    with_f32: Query<With<f32, &mut u32>>,
    without_f32: Query<Without<f32, &mut u32>>,
) {
    yaks::batch(&mut with_f32.query(), entities / 8, |_entity, unsigned| {
        *unsigned += average.round() as u32;
    });
    yaks::batch(
        &mut without_f32.query(),
        entities / 8,
        |_entity, unsigned| {
            *unsigned *= average.round() as u32;
        },
    );
}

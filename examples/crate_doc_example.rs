//! Duplication of the crate level documentation example, for ease of editing.

use hecs::World;
use resources::Resources;
use yaks::{Executor, ModQueuePool, System};

struct Position(f32);
struct Velocity(f32);
struct Acceleration(f32);
struct HighestVelocity(f32);

fn main() {
    let mut world = World::new();
    let mut resources = Resources::new();
    let mod_queues = ModQueuePool::new();
    world.spawn((Position(0.0), Velocity(3.0)));
    world.spawn((Position(0.0), Velocity(1.0), Acceleration(1.0)));
    resources.insert(HighestVelocity(0.0));

    let motion = System::builder()
        .query::<(&mut Position, &Velocity)>()
        .query::<(&mut Velocity, &Acceleration)>()
        .build(|facade, _, (q_1, q_2)| {
            for (_, (mut pos, vel)) in facade.query(q_1).iter() {
                pos.0 += vel.0;
            }
            for (_, (mut vel, acc)) in facade.query(q_2).iter() {
                vel.0 += acc.0;
            }
        });

    let find_highest = System::builder()
        .resources::<&mut HighestVelocity>()
        .query::<&Velocity>()
        .build(|facade, mut highest, query| {
            for (_, vel) in facade.query(query).iter() {
                if vel.0 > highest.0 {
                    highest.0 = vel.0;
                }
            }
        });

    let mut executor = Executor::<()>::builder()
        .system(motion)
        .system(find_highest)
        .build();
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 0.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 4.0);
}

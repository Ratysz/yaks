//! Duplication of the crate level documentation example, for ease of editing.

use yaks::{Executor, ModQueuePool, Resources, System, World};

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
        .build(|world, _, _, _, (q_1, q_2)| {
            for (_, (mut pos, vel)) in q_1.query(world).iter() {
                pos.0 += vel.0;
            }
            for (_, (mut vel, acc)) in q_2.query(world).iter() {
                vel.0 += acc.0;
            }
        });

    let find_highest = System::builder()
        .resources::<&mut HighestVelocity>()
        .query::<&Velocity>()
        .build(|world, _, _, mut highest, query| {
            for (_, vel) in query.query(world).iter() {
                if vel.0 > highest.0 {
                    highest.0 = vel.0;
                }
            }
        });

    let mut executor = Executor::<()>::new().with(motion).with(find_highest);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 0.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 4.0);
}

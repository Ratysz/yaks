//! An intentionally convoluted (and inefficient) example, simulating a race between
//! three entities, complete with celebratory confetti.

use yaks::{Entity, Executor, System, World};

struct Position(f32);

struct Velocity(f32);

struct HasFinished(bool);

struct FinishLine(f32);

struct Iteration(usize);

struct Winner(Entity);

struct ConfettiTimer(i32);

fn main() {
    let mut world = World::new();
    world.spawn((Position(0.0), Velocity(3.0), HasFinished(false)));
    world.spawn((Position(5.0), Velocity(2.0), HasFinished(false)));
    world.spawn((Position(10.0), Velocity(1.0), HasFinished(false)));
    world.add_resource(Iteration(0));
    world.add_resource(FinishLine(50.0));

    let racing = System::builder()
        .resources::<&FinishLine>()
        .query::<(&mut Position, &Velocity)>()
        .query::<(&Position, &mut HasFinished)>()
        .build(|world, finish_line, (query_1, query_2)| {
            for (_, (mut position, velocity)) in query_1.query(world).iter() {
                position.0 += velocity.0;
            }
            for (entity, (position, mut finished)) in query_2.query(world).iter() {
                if position.0 >= finish_line.0 {
                    finished.0 = true;
                    if !world.contains_resource::<Winner>() {
                        world.new_mod_queue().push(move |world| {
                            world.add_resource(Winner(entity));
                        });
                    }
                }
            }
        });

    let leave_track =
        System::builder()
            .query::<&HasFinished>()
            .build(|world, _, query| {
                let mut queue = world.new_mod_queue();
                for entity in query.query(world).iter().filter_map(|(entity, finished)| {
                    if finished.0 {
                        Some(entity)
                    } else {
                        None
                    }
                }) {
                    queue.push(move |world| {
                        world.despawn(entity).unwrap();
                    });
                }
            });

    let stopwatch = System::builder()
        .resources::<&mut Iteration>()
        .build(|_, mut iteration, _| {
            iteration.0 += 1;
        });

    let spawn_confetti = System::builder()
        .query::<&ConfettiTimer>()
        .build(|world, _, query| {
            if query.query(world).iter().len() < 20 {
                world.new_mod_queue().push(|world| {
                    world.spawn((ConfettiTimer(50),));
                    world.spawn((ConfettiTimer(40),));
                    world.spawn((ConfettiTimer(30),));
                })
            }
        });

    let confetti_cleanup =
        System::builder()
            .query::<&mut ConfettiTimer>()
            .build(|world, _, query| {
                for (_, mut timer) in query.query(world).iter() {
                    timer.0 -= 1;
                }
                let decayed_confetti = query
                    .query(world)
                    .iter()
                    .filter_map(
                        |(entity, timer)| {
                            if timer.0 < 0 {
                                Some(entity)
                            } else {
                                None
                            }
                        },
                    )
                    .collect::<Vec<_>>();
                world.new_mod_queue().push(move |world| {
                    for entity in &decayed_confetti {
                        world.despawn(*entity).unwrap();
                    }
                });
            });

    let mut executor = Executor::<&'static str>::new()
        .with_handle("racing", racing)
        .with(leave_track)
        .with_handle("stopwatch", stopwatch)
        .with_handle("confetti", spawn_confetti)
        .with(confetti_cleanup);

    executor.set_active(&"confetti", false).unwrap();

    print!("Turn |");
    for entity in world.query::<&Position>().iter().map(|(entity, _)| entity) {
        print!("{:?}|", entity);
    }
    println!();
    while !world.contains_resource::<Winner>() {
        let stopwatch = world.fetch::<&Iteration>().0;
        print!("  {:2} |", stopwatch);
        for (_, position) in world.query::<&Position>().iter() {
            print!("{:3}|", position.0);
        }
        println!();
        executor.run(&mut world);
    }

    println!();
    println!("The winner is {:?}!", world.resource::<Winner>().unwrap().0);

    executor.set_active(&"confetti", true).unwrap();
    executor.set_active(&"stopwatch", false).unwrap();
    executor.set_active(&"racing", false).unwrap();

    for i in 0..60 {
        executor.run(&mut world);
        for (_, timer) in world.query::<&ConfettiTimer>().iter() {
            if timer.0 > 30 {
                print!("'");
            } else if timer.0 > 15 {
                print!("*");
            } else {
                print!(".");
            }
        }
        println!();
        if i == 30 {
            executor.set_active(&"confetti", false).unwrap();
        }
    }
}

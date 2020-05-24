//! An intentionally convoluted (and inefficient) example, simulating a race between
//! three entities, complete with celebratory confetti.

fn main() {}
/*
use hecs::{Entity, World};
use resources::Resources;
use yaks::{Executor, ModQueuePool, System};

struct Position(f32);

struct Velocity(f32);

struct HasFinished(bool);

struct FinishLine(f32);

struct Iteration(usize);

struct Winner(Entity);

struct ConfettiTimer(i32);

fn main() {
    let mut world = World::new();
    let mut resources = Resources::new();
    let mod_queues = ModQueuePool::new();
    world.spawn((Position(0.0), Velocity(3.0), HasFinished(false)));
    world.spawn((Position(5.0), Velocity(2.0), HasFinished(false)));
    world.spawn((Position(10.0), Velocity(1.0), HasFinished(false)));
    resources.insert(Iteration(0));
    resources.insert(FinishLine(50.0));

    let racing = System::builder()
        .resources::<&FinishLine>()
        .query::<(&mut Position, &Velocity)>()
        .query::<(&Position, &mut HasFinished)>()
        .build(|facade, finish_line, (query_1, query_2)| {
            for (_, (mut position, velocity)) in facade.query(query_1).iter() {
                position.0 += velocity.0;
            }
            for (entity, (position, mut finished)) in facade.query(query_2).iter() {
                if position.0 >= finish_line.0 {
                    finished.0 = true;
                    if !facade.resources.contains::<Winner>() {
                        facade.new_mod_queue().push(move |_, resources| {
                            resources.insert(Winner(entity));
                        });
                    }
                }
            }
        });

    let leave_track =
        System::builder()
            .query::<&HasFinished>()
            .build(|facade, _, query| {
                let mut queue = facade.new_mod_queue();
                for entity in facade.query(query).iter().filter_map(|(entity, finished)| {
                    if finished.0 {
                        Some(entity)
                    } else {
                        None
                    }
                }) {
                    queue.push(move |world, _| {
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
        .build(|facade, _, query| {
            if facade.query(query).iter().len() < 20 {
                facade.new_mod_queue().push(|world, _| {
                    world.spawn((ConfettiTimer(50),));
                    world.spawn((ConfettiTimer(40),));
                    world.spawn((ConfettiTimer(30),));
                })
            }
        });

    let confetti_cleanup =
        System::builder()
            .query::<&mut ConfettiTimer>()
            .build(|facade, _, query| {
                for (_, mut timer) in facade.query(query).iter() {
                    timer.0 -= 1;
                }
                let decayed_confetti = facade
                    .query(query)
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
                facade.new_mod_queue().push(move |world, _| {
                    for entity in &decayed_confetti {
                        world.despawn(*entity).unwrap();
                    }
                });
            });

    let mut executor = Executor::<&'static str>::builder()
        .system_with_handle(racing, "racing")
        .system(leave_track)
        .system_with_handle(stopwatch, "stopwatch")
        .system_with_handle(spawn_confetti, "confetti")
        .system(confetti_cleanup)
        .build();

    executor.set_active(&"confetti", false).unwrap();

    print!("Turn |");
    for entity in world.query::<&Position>().iter().map(|(entity, _)| entity) {
        print!("{:?}|", entity);
    }
    println!();
    while !resources.contains::<Winner>() {
        let stopwatch = resources.get::<Iteration>().unwrap().0;
        print!("  {:2} |", stopwatch);
        for (_, position) in world.query::<&Position>().iter() {
            print!("{:3}|", position.0);
        }
        println!();
        executor.run(&world, &resources, &mod_queues);
        mod_queues.apply_all(&mut world, &mut resources);
    }

    println!();
    println!("The winner is {:?}!", resources.get::<Winner>().unwrap().0);

    executor.set_active(&"confetti", true).unwrap();
    executor.set_active(&"stopwatch", false).unwrap();
    executor.set_active(&"racing", false).unwrap();

    for i in 0..60 {
        executor.run(&world, &resources, &mod_queues);
        mod_queues.apply_all(&mut world, &mut resources);
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
*/

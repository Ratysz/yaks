use yaks::next::{Executor, QueryMarker, SystemContext};

#[derive(Debug)]
struct A(usize);
#[derive(Debug)]
struct B(usize);
#[derive(Debug)]
struct C(usize);

fn some_system(context: SystemContext, (a, b): (&mut A, &B), query0: QueryMarker<(&mut B, &C)>) {
    std::thread::sleep(std::time::Duration::from_millis(15));
    print!("{} + {} = ", a.0, b.0);
    a.0 += b.0;
    println!("{}", a.0);
    for (entity, (mut b, c)) in context.query(query0).iter() {
        b.0 += c.0;
        println!("entity {:?} b: {}", entity, b.0);
    }
    std::thread::sleep(std::time::Duration::from_millis(5));
}

#[test]
fn test() {
    let mut world = hecs::World::new();
    world.spawn((A(0), B(0), C(1)));
    world.spawn((B(0), C(1)));
    let mut a = A(0);
    let mut b = B(1);
    let mut c = C(2);

    let mut executor: Executor<(A, B, C)> = Executor::builder()
        .system(|context, _: &B, _: QueryMarker<&mut A>| {
            println!("{:?} &B, QueryMarker<&mut A>", context.system_id);
            std::thread::sleep(std::time::Duration::from_millis(10));
        })
        .system_with_handle(
            |context, _: &mut C, _: QueryMarker<(&B, &mut C)>| {
                println!(
                    "{:?} &mut C, QueryMarker<&mut B, &mut C>, 'system_with_C'",
                    context.system_id
                );
                std::thread::sleep(std::time::Duration::from_millis(10));
            },
            "system_with_C",
        )
        .system(|context, _: &B, _: QueryMarker<Option<&C>>| {
            println!("{:?} &B, QueryMarker<Option<&C>>", context.system_id);
            std::thread::sleep(std::time::Duration::from_millis(10));
        })
        .system(|context, _: &B, _: QueryMarker<Option<&B>>| {
            println!("{:?} &B, QueryMarker<Option<&B>>", context.system_id);
            std::thread::sleep(std::time::Duration::from_millis(10));
        })
        .system_with_deps(
            |context, _: (&mut A, &B), _: QueryMarker<(&mut B, &C)>| {
                println!(
                    "{:?} &mut A, &B, QueryMarker<&mut B, &C>, depends on 'system_with_C'",
                    context.system_id
                );
                std::thread::sleep(std::time::Duration::from_millis(10));
            },
            vec!["system_with_C"],
        )
        .build();

    executor.run(&world, (&mut a, &mut b, &mut c));
    println!();
    executor.run(&world, (&mut a, &mut b, &mut c));
}

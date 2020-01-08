use secs::{System, World};

struct ResUsize(usize);

struct ResStr(&'static str);

struct Comp1(usize);

struct Comp2(usize);

struct Comp3(usize);

fn main() {
    let mut world = World::new();
    world.add_resource(ResUsize(1));
    world.add_resource(ResStr("Hello!"));
    let world = world;

    {
        let (one, mut two) = world.fetch::<(&ResUsize, &mut ResStr)>();
        two.0 = "Bye!";
        assert_eq!(one.0, 1);
    }

    {
        let (mut one, two) = world.fetch::<(&mut ResUsize, &ResStr)>();
        one.0 += 1;
        assert_eq!(two.0, "Bye!");
    }

    assert_eq!(world.fetch::<&ResUsize>().0, 2);

    let mut world = world;
    world.spawn((Comp1(1), Comp2(0)));
    world.spawn((Comp1(1),));
    world.spawn((Comp3(1),));
    world.spawn((Comp1(1), Comp2(0), Comp3(0)));
    let world = world;

    let increment = 6;

    let mut system = System::builder()
        .resource::<&ResUsize>()
        .resource::<&mut ResStr>()
        .query::<(&mut Comp1, &Comp2)>()
        .query::<&Comp1>()
        .query::<(&mut Comp1, Option<&Comp2>)>()
        .build(
            move |world, (res_usize, mut res_str), (query1, query2, query3)| {
                res_str.0 = "Hello again!";
                for (_, (mut comp1, comp2)) in query1.query(world).into_iter() {
                    comp1.0 += increment;
                }
            },
        );
    system.run(&world);

    assert_eq!(world.fetch::<&ResStr>().0, "Hello again!");

    System::builder()
        .resource::<&ResUsize>()
        .query::<&Comp3>()
        .build(move |world, res_usize, q| {
            q.query(world);
        });

    System::builder()
        .query::<&Comp3>()
        .query::<&mut Comp2>()
        .build(move |world, _, (q1, q2)| {
            q1.query(world);
            q2.query(world);
        });

    System::builder()
        .query::<(&Comp3, &mut Comp2)>()
        .build(
            move |world, _, q| {
                for (_, (_, _)) in q.query(world).into_iter() {}
            },
        );
}

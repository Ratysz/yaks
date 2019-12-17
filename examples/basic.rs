use secs::{DynamicSystemBuilder, StaticSystem, StaticSystemBuilder, World};

struct ResourceOne(usize);

struct ResourceTwo(&'static str);

struct ComponentOne(usize);

struct ComponentTwo(usize);

fn main() {
    let mut world = World::new();
    world.add_resource(ResourceOne(1));
    world.add_resource(ResourceTwo("Hello!"));
    let world = world;

    {
        let (one, mut two) = world.fetch::<(&ResourceOne, &mut ResourceTwo)>();
        two.0 = "Bye!";
        assert_eq!(one.0, 1);
    }

    {
        let (mut one, two) = world.fetch::<(&mut ResourceOne, &ResourceTwo)>();
        one.0 += 1;
        assert_eq!(two.0, "Bye!");
    }

    assert_eq!(world.fetch::<&ResourceOne>().0, 2);

    let mut world = world;
    world.spawn((ComponentOne(1), ComponentTwo(0)));
    let world = world;

    let increment = 6;

    let mut system1 = StaticSystemBuilder::<
        (&ResourceOne, &mut ResourceTwo),
        (
            (&mut ComponentOne, &ComponentTwo),
            Option<&ComponentOne>,
            &mut ComponentOne,
        ),
    >::build(move |world, (r_one, mut r_two), (query1, query2, _)| {
        r_two.0 = "Hello again!";
        for (_, (mut one, two)) in query1.query(world).into_iter() {
            one.0 += increment;
        }
        for (_, one) in query2.query(world).into_iter() {
            if let Some(one) = one {
                assert_eq!(one.0, 7);
            }
        }
    });
    system1.run(&world);

    let mut system2 = DynamicSystemBuilder::<
        (&ResourceOne, &mut ResourceTwo),
        (
            (&mut ComponentOne, &ComponentTwo),
            Option<&ComponentOne>,
            &mut ComponentOne,
        ),
    >::build(move |world, (r_one, mut r_two), (query1, query2, _)| {
        r_two.0 = "Hello again!";
        for (_, (mut one, two)) in query1.query(world).into_iter() {
            one.0 += increment;
        }
        for (_, one) in query2.query(world).into_iter() {
            if let Some(one) = one {
                assert_eq!(one.0, 13);
            }
        }
    });
    system2.run(&world);

    for type_id in system1.borrowed_components() {
        println!("read: {:?}", type_id);
    }

    for type_id in system1.borrowed_mut_components() {
        println!("write: {:?}", type_id);
    }

    /*for type_id in system2.borrowed_components() {
        println!("read: {:?}", type_id);
    }

    for type_id in system2.borrowed_mut_components() {
        println!("write: {:?}", type_id);
    }*/

    assert_eq!(world.fetch::<&ResourceTwo>().0, "Hello again!");
}

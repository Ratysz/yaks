use secs::{System, World};

struct ResourceOne(usize);

struct ResourceTwo(&'static str);

struct ComponentOne(usize);

struct ComponentTwo(usize);

struct ComponentThree(usize);

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
    world.spawn((ComponentOne(1),));
    world.spawn((ComponentThree(1),));
    let world = world;

    let increment = 6;

    let mut system = System::<
        (&ResourceOne, &mut ResourceTwo),
        (
            (&mut ComponentOne, &ComponentTwo),
            Option<&ComponentOne>,
            &mut ComponentOne,
        ),
    >::build(
        move |world, (resource_1, mut resource_2), (query_1, query_2, query_3)| {
            resource_2.0 = "Hello again!";
            for (_, (mut component_1, component_2)) in query_1.query(world).into_iter() {
                component_1.0 += increment;
            }
        },
    );
    system.run(&world);

    let mut system = System::<&ResourceTwo, ((&mut ComponentThree, &ComponentThree),)>::build(
        move |world, resource_2, q1| {},
    );

    for id in system.touched_archetypes(&world) {
        println!("archetype: {:?}", id);
    }
    println!();

    println!("resources");
    for type_id in system.borrowed_resources() {
        println!(" read: {:?}", type_id);
    }
    for type_id in system.borrowed_mut_resources() {
        println!(" write: {:?}", type_id);
    }
    println!();

    println!("components");
    for type_id in system.borrowed_components() {
        println!(" read: {:?}", type_id);
    }
    for type_id in system.borrowed_mut_components() {
        println!(" write: {:?}", type_id);
    }

    assert_eq!(world.fetch::<&ResourceTwo>().0, "Hello again!");
}

use secs::{SystemBuilder, World};

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

    let mut system = SystemBuilder::<
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

    let mut system =
        SystemBuilder::<&ResourceTwo, ((&mut ComponentThree, &ComponentThree),)>::build(
            move |world, resource_2, q1| {},
        );

    let mut archetypes = Default::default();
    system.write_touched_archetypes(&world, &mut archetypes);
    for id in &archetypes {
        println!("archetype: {:?}", id);
    }
    println!();

    println!("metadata: {:#?}", system.metadata());

    assert_eq!(world.fetch::<&ResourceTwo>().0, "Hello again!");
}

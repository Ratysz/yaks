use secs::{System, SystemBuilderDynamic, SystemBuilderStatic, World};

struct One(usize);

struct Two(&'static str);

fn main() {
    let mut world = World::new();
    world.add_resource(One(1));
    world.add_resource(Two("Hello!"));
    let world = world;

    {
        let (one, mut two) = world.fetch::<(&One, &mut Two)>();
        two.0 = "Bye!";
        assert_eq!(one.0, 1);
    }

    {
        let (mut one, two) = world.fetch::<(&mut One, &Two)>();
        one.0 += 1;
        assert_eq!(two.0, "Bye!");
    }

    assert_eq!(world.fetch::<&One>().0, 2);

    let mut world = world;
    world.spawn((One(1), Two("")));
    let world = world;

    let increment = 6;

    let mut system1 = SystemBuilderStatic::<
        (&One, &mut Two),
        ((&mut One, &Two), Option<&One>, &mut One),
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

    let mut system2 = SystemBuilderDynamic::<
        (&One, &mut Two),
        ((&mut One, &Two), Option<&One>, &mut One),
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

    assert_eq!(world.fetch::<&Two>().0, "Hello again!");
}

use secs::*;

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

    assert_eq!(world.resource::<One>().unwrap().0, 2);
}

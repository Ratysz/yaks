use hecs::World;
use yaks::{Executor, MarkerGet, Mut, Ref};

#[derive(Default)]
struct Resources {
    a: std::cell::RefCell<usize>,
    b: f32,
}

impl<'a> MarkerGet<&'a Resources> for Mut<usize> {
    type Fetched = std::cell::RefMut<'a, usize>;

    fn fetch(source: &'a Resources) -> Self::Fetched {
        source.a.borrow_mut()
    }
}

impl<'a> MarkerGet<&'a Resources> for Ref<f32> {
    type Fetched = &'a f32;

    fn fetch(source: &'a Resources) -> Self::Fetched {
        &source.b
    }
}

fn main() {
    let world = World::new();
    let resources = Resources::default();

    let mut executor = Executor::<(Mut<usize>, Ref<f32>)>::builder().build();
    executor.run(&world, &resources);
}

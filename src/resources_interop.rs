use hecs::World;
use resources::{Resource, Resources};

use crate::{Executor, ResourceTuple};

pub trait InvertedWrap: ResourceTuple + Sized {
    fn run(executor: &mut Executor<Self>, world: &World, resources: &Resources);
}

impl<R0> InvertedWrap for (R0,)
where
    R0: Resource,
{
    fn run(executor: &mut Executor<Self>, world: &World, resources: &Resources) {
        let mut refs = (resources.get_mut::<R0>().unwrap(),);
        let derefs = (&mut *refs.0,);
        executor.run(world, derefs);
    }
}

macro_rules! impl_scoped_fetch {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> InvertedWrap for ($($letter,)*)
        where
            $($letter: Resource,)*
        {
            #[allow(non_snake_case)]
            fn run(executor: &mut Executor<Self>, world: &World, resources: &Resources) {
                let ($(mut $letter,)*) = ($(resources.get_mut::<$letter>().unwrap(),)*);
                let derefs = ($(&mut *$letter,)*);
                executor.run(world, derefs);
            }
        }
    }
}

impl_for_tuples!(impl_scoped_fetch);

#[test]
fn smoke_test() {
    use crate::Executor;
    let world = hecs::World::new();
    let mut resources = resources::Resources::new();
    resources.insert(1.0f32);
    resources.insert(2u32);
    resources.insert(3u64);
    let mut executor = Executor::<(f32, u32, u64)>::builder()
        .system(|_, _: (&mut f32, &u32), _: ()| {})
        .system(|_, _: (&mut f32, &u64), _: ()| {})
        .build();
    executor.run_with_dyn_resources(&world, &resources);
}

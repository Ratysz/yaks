use hecs::World;
use resources::{Resource, Resources};

use crate::{Executor, RefExtractor};

impl RefExtractor<&Resources> for () {
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, _: &Resources) {
        executor.run(world, ());
    }
}

impl<R0> RefExtractor<&Resources> for (R0,)
where
    R0: Resource,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: &Resources) {
        let mut refs = resources
            .fetch::<&mut R0>()
            .unwrap_or_else(|error| panic!("{}", error));
        let derefs = (&mut *refs,);
        executor.run(world, derefs);
    }
}

macro_rules! impl_ref_extractor {
    ($($letter:ident),*) => {
        impl<'a, $($letter),*> RefExtractor<&Resources> for ($($letter,)*)
        where
            $($letter: Resource,)*
        {
            #[allow(non_snake_case)]
            fn extract_and_run(
                executor: &mut Executor<Self>,
                world: &World,
                resources: &Resources,
            ) {
                let ($(mut $letter,)*) = resources
                    .fetch::<($(&mut $letter, )*)>()
                    .unwrap_or_else(|error| panic!("{}", error));
                let derefs = ($(&mut *$letter,)*);
                executor.run(world, derefs);
            }
        }
    }
}

impl_for_tuples!(impl_ref_extractor);

#[test]
fn smoke_test() {
    use crate::Executor;
    let mut executor = Executor::<(f32, u32, u64)>::builder()
        .system(|_, _: (&mut f32, &u32), _: ()| {})
        .system(|_, _: (&mut f32, &u64), _: ()| {})
        .build();
    let world = hecs::World::new();

    let (mut a, mut b, mut c) = (1.0f32, 2u32, 3u64);
    executor.run(&world, (&mut a, &mut b, &mut c));

    let mut resources = resources::Resources::new();
    resources.insert(1.0f32);
    resources.insert(2u32);
    resources.insert(3u64);
    executor.run(&world, &resources);
}

use std::marker::PhantomData;

use crate::{
    metadata::ArchetypeSet,
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    SystemMetadata, World,
};

pub trait System {
    fn run(&mut self, world: &World);

    fn write_metadata(&self, metadata: &mut SystemMetadata);

    fn write_touched_archetypes(&self, world: &World, set: &mut ArchetypeSet);
}

struct SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(&World, R::Effectors, Q::Effectors)>,
}

impl<R, Q> System for SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(world, R::effectors(), Q::effectors())
    }

    fn write_metadata(&self, metadata: &mut SystemMetadata) {
        R::write_metadata(metadata);
        Q::write_metadata(metadata);
    }

    fn write_touched_archetypes(&self, world: &World, set: &mut ArchetypeSet) {
        Q::write_touched_archetypes(world, set);
    }
}

pub struct SystemBuilder<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> SystemBuilder<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    pub fn build<'a, F>(mut closure: F) -> Box<dyn System>
    where
        R::Effectors: Fetch<'a>,
        F: FnMut(&'a World, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    {
        let closure = Box::new(move |world, resources: R::Effectors, queries| {
            closure(world, resources.fetch(world), queries)
        });
        let closure = unsafe {
            // FIXME this is a dirty hack for until
            //  https://github.com/rust-lang/rust/issues/62529 is fixed
            // I'm all but certain that this, within the context it's used in, is safe.
            // This transmutation forces the compiler to accept lifetime bounds it would've been
            // able to verify itself, if they were written as a HRTB.
            // Since HRTBs cause an ICE when used with closures in the way that's needed here
            // (see link above), I've opted for this workaround.
            std::mem::transmute::<
                Box<dyn FnMut(&'a World, R::Effectors, Q::Effectors)>,
                Box<dyn FnMut(&World, R::Effectors, Q::Effectors)>,
            >(closure)
        };
        Box::new(SystemBox::<R, Q> {
            phantom_data: PhantomData,
            closure,
        })
    }
}

#[test]
fn test() {
    let mut world = World::new();
    world.add_resource::<usize>(1);
    world.add_resource::<f32>(1.0);
    let mut system = SystemBuilder::<(&usize, &mut f32), (&usize, Option<&usize>)>::build(
        |world, (res1, mut res2), query| {
            *res2 += 1.0;
        },
    );
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

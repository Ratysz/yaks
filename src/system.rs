use std::{borrow::Cow, marker::PhantomData};

use crate::{
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeSet, TypeSet, World,
};

pub trait System {
    fn run(&mut self, world: &World);

    fn metadata(&self) -> Cow<SystemMetadata>;

    fn write_touched_archetypes(&self, world: &World, set: &mut ArchetypeSet);
}

#[derive(Default, Debug, Clone)]
pub struct SystemMetadata {
    pub resources: TypeSet,
    pub resources_mut: TypeSet,
    pub components: TypeSet,
    pub components_mut: TypeSet,
}

type DynamicSystemClosure<R, Q> =
    Box<dyn FnMut(&World, <R as ResourceBundle>::Effectors, <Q as QueryBundle>::Effectors)>;

struct DynamicSystem<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
    metadata: SystemMetadata,
    closure: DynamicSystemClosure<R, Q>,
}

impl<R, Q> DynamicSystem<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn new(closure: DynamicSystemClosure<R, Q>) -> Self {
        let mut metadata = SystemMetadata::default();
        R::write_metadata(&mut metadata);
        Q::write_metadata(&mut metadata);
        Self {
            phantom_data: PhantomData,
            metadata,
            closure,
        }
    }
}

impl<R, Q> System for DynamicSystem<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(world, R::effectors(), Q::effectors())
    }

    fn metadata(&self) -> Cow<SystemMetadata> {
        Cow::Borrowed(&self.metadata)
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
    pub fn build<'a, F>(closure: F) -> Box<dyn System>
    where
        R::Effectors: Fetch<'a>,
        F: FnMut(&'a World, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    {
        Box::new(DynamicSystem::<R, Q>::new(Self::transmute_closure(closure)))
    }

    fn transmute_closure<'a, F>(mut closure: F) -> DynamicSystemClosure<R, Q>
    where
        R::Effectors: Fetch<'a>,
        F: FnMut(&'a World, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    {
        let closure = Box::new(move |world, resources: R::Effectors, queries| {
            closure(world, resources.fetch(world), queries)
        });
        unsafe {
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
        }
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

use std::{iter::FromIterator, marker::PhantomData};

use crate::{
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeSet, TypeSet, World,
};

type SystemClosure<R, Q> =
    Box<dyn FnMut(&World, <R as ResourceBundle>::Effectors, <Q as QueryBundle>::Effectors)>;

pub trait DynamicSystem {
    fn run(&mut self, world: &World);

    fn touched_archetypes(&mut self, world: &World) -> &ArchetypeSet;

    fn borrowed_components(&self) -> &TypeSet;

    fn borrowed_mut_components(&self) -> &TypeSet;

    fn borrowed_resources(&self) -> &TypeSet;

    fn borrowed_mut_resources(&self) -> &TypeSet;
}

pub trait SystemTrait {
    fn run(&mut self, world: &World);
}

impl<S: DynamicSystem> SystemTrait for S {
    fn run(&mut self, world: &World) {
        self.run(world)
    }
}

struct SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
    closure: SystemClosure<R, Q>,
    archetypes: ArchetypeSet,
    components: TypeSet,
    components_mut: TypeSet,
    resources: TypeSet,
    resources_mut: TypeSet,
}

impl<R, Q> SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn new(closure: SystemClosure<R, Q>) -> Self {
        let mut components = TypeSet::default();
        let mut components_mut = TypeSet::default();
        let mut resources = TypeSet::default();
        let mut resources_mut = TypeSet::default();
        Q::write_borrowed_components(&mut components);
        Q::write_borrowed_mut_components(&mut components_mut);
        R::write_borrowed_resources(&mut resources);
        R::write_borrowed_mut_resources(&mut resources_mut);
        Self {
            phantom_data: PhantomData,
            closure,
            archetypes: ArchetypeSet::default(),
            components,
            components_mut,
            resources,
            resources_mut,
        }
    }
}

impl<R, Q> DynamicSystem for SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(world, R::effectors(), Q::effectors())
    }

    fn touched_archetypes(&mut self, world: &World) -> &ArchetypeSet {
        world.write_touched_archetypes_if_invalidated::<Q>(&mut self.archetypes);
        &self.archetypes
    }

    fn borrowed_components(&self) -> &TypeSet {
        &self.components
    }

    fn borrowed_mut_components(&self) -> &TypeSet {
        &self.components_mut
    }

    fn borrowed_resources(&self) -> &TypeSet {
        &self.resources
    }

    fn borrowed_mut_resources(&self) -> &TypeSet {
        &self.resources_mut
    }
}

pub struct System<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> System<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    pub fn build<'a, F>(closure: F) -> Box<dyn DynamicSystem>
    where
        R::Effectors: Fetch<'a>,
        F: FnMut(&'a World, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    {
        Box::new(SystemBox::<R, Q>::new(Self::transmute_closure(closure)))
    }

    fn transmute_closure<'a, F>(mut closure: F) -> SystemClosure<R, Q>
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
    let mut system = System::<(&usize, &mut f32), (&usize, Option<&usize>)>::build(
        |world, (res1, mut res2), query| {
            *res2 += 1.0;
        },
    );
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

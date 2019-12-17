use std::{iter::FromIterator, marker::PhantomData};

use crate::{
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeSet, TypeSet, World,
};

pub trait DynamicSystem {
    fn run(&mut self, world: &World);

    fn borrowed_components(&self) -> TypeSet;

    fn borrowed_mut_components(&self) -> TypeSet;

    fn touched_archetypes(&self, world: &World) -> ArchetypeSet;

    fn borrowed_resources(&self) -> TypeSet;

    fn borrowed_mut_resources(&self) -> TypeSet;
}

impl<R, Q> DynamicSystem
    for (
        PhantomData<(R, Q)>,
        Box<dyn FnMut(&World, R::Effectors, Q::Effectors)>,
    )
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.1)(world, R::effectors(), Q::effectors())
    }

    fn borrowed_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_components())
    }

    fn borrowed_mut_components(&self) -> TypeSet {
        TypeSet::from_iter(Q::borrowed_mut_components())
    }

    fn touched_archetypes(&self, world: &World) -> ArchetypeSet {
        Q::touched_archetypes(world)
    }

    fn borrowed_resources(&self) -> TypeSet {
        TypeSet::from_iter(R::borrowed_resources())
    }

    fn borrowed_mut_resources(&self) -> TypeSet {
        TypeSet::from_iter(R::borrowed_mut_resources())
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
    pub fn new<'a>(
        closure: impl FnMut(&'a World, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    ) -> Box<dyn DynamicSystem>
    where
        R::Effectors: Fetch<'a>,
    {
        Box::new((PhantomData::<(R, Q)>, Self::transmute_closure(closure)))
    }

    fn transmute_closure<'a, F>(
        mut closure: F,
    ) -> Box<dyn FnMut(&World, R::Effectors, Q::Effectors)>
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
    let mut system = System::<(&usize, &mut f32), (&usize, Option<&usize>)>::new(
        |world, (res1, mut res2), query| {
            *res2 += 1.0;
        },
    );
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

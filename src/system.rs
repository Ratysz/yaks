use std::marker::PhantomData;

use crate::{
    borrows::{ArchetypeSet, SystemBorrows},
    impls_for_tuple::TupleAppend,
    query_bundle::{QueryBundle, QuerySingle, QueryUnit},
    resource_bundle::{Fetch, ResourceBundle, ResourceSingle},
    World, WorldProxy,
};

pub trait SystemTrait {
    fn run(&mut self, world: &World);

    fn write_borrows(&self, borrows: &mut SystemBorrows);

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet);
}

struct SystemBox<RB, QB, CB>
where
    RB: ResourceBundle,
    QB: QueryBundle,
    CB: QueryBundle,
{
    phantom_data: PhantomData<(RB, QB, CB)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(WorldProxy, RB::Effectors, QB::Effectors)>,
}

impl<RB, QB, CB> SystemTrait for SystemBox<RB, QB, CB>
where
    RB: ResourceBundle,
    QB: QueryBundle,
    CB: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(WorldProxy::new(world), RB::effectors(), QB::effectors())
    }

    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        RB::write_borrows(borrows);
        QB::write_borrows(borrows);
        CB::write_borrows(borrows);
    }

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet) {
        QB::write_archetypes(world, archetypes);
        CB::write_archetypes(world, archetypes);
    }
}

pub struct System {
    pub(crate) inner: Box<dyn SystemTrait>,
}

impl System {
    pub fn builder() -> SystemBuilder<(), (), ()> {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn run(&mut self, world: &World) {
        self.inner.run(world);
    }
}

pub struct SystemBuilder<RB, QB, CB>
where
    RB: ResourceBundle + 'static,
    QB: QueryBundle + 'static,
    CB: QueryBundle + 'static,
{
    phantom_data: PhantomData<(RB, QB, CB)>,
}

impl<RB, QB, CB> SystemBuilder<RB, QB, CB>
where
    RB: ResourceBundle + 'static,
    QB: QueryBundle + 'static,
    CB: QueryBundle + 'static,
{
    pub fn resource<R>(self) -> SystemBuilder<RB::Output, QB, CB>
    where
        R: ResourceSingle,
        RB: TupleAppend<R>,
        <RB as TupleAppend<R>>::Output: ResourceBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn query<Q>(self) -> SystemBuilder<RB, QB::Output, CB>
    where
        Q: QuerySingle,
        QB: TupleAppend<Q>,
        <QB as TupleAppend<Q>>::Output: QueryBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn component<C>(self) -> SystemBuilder<RB, QB, CB::Output>
    where
        C: QueryUnit,
        CB: TupleAppend<C>,
        <CB as TupleAppend<C>>::Output: QueryBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn build<'a, F>(self, mut closure: F) -> System
    where
        RB::Effectors: Fetch<'a>,
        F: FnMut(WorldProxy<'a>, <RB::Effectors as Fetch<'a>>::Refs, QB::Effectors) + 'static,
    {
        let closure = Box::new(move |proxy, resources: RB::Effectors, queries| {
            closure(proxy, resources.fetch(proxy.world), queries)
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
                Box<dyn FnMut(WorldProxy<'a>, RB::Effectors, QB::Effectors)>,
                Box<dyn FnMut(WorldProxy, RB::Effectors, QB::Effectors)>,
            >(closure)
        };
        System {
            inner: Box::new(SystemBox::<RB, QB, CB> {
                phantom_data: PhantomData,
                closure,
            }),
        }
    }
}

#[test]
fn test() {
    let mut world = World::new();
    world.add_resource::<usize>(1);
    world.add_resource::<f32>(1.0);
    let mut system = System::builder()
        .resource::<&usize>()
        .resource::<&mut f32>()
        .query::<(&usize, Option<&usize>)>()
        .build(|world, (res1, mut res2), query| {
            *res2 += 1.0;
        });
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

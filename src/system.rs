use std::marker::PhantomData;

use crate::{
    borrows::{ArchetypeSet, SystemBorrows},
    query_bundle::QueryBundle,
    resource_bundle::{Fetch, ResourceBundle},
    World, WorldProxy,
};

pub trait SystemTrait {
    fn run(&mut self, world: &World);

    fn write_borrows(&self, borrows: &mut SystemBorrows);

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet);
}

struct SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    phantom_data: PhantomData<(R, Q)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(WorldProxy, R::Effectors, Q::Effectors)>,
}

impl<R, Q> SystemTrait for SystemBox<R, Q>
where
    R: ResourceBundle,
    Q: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(WorldProxy::new(world), R::effectors(), Q::effectors())
    }

    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        R::write_borrows(borrows);
        Q::write_borrows(borrows);
    }

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet) {
        Q::write_archetypes(world, archetypes);
    }
}

pub struct System {
    pub(crate) inner: Box<dyn SystemTrait>,
}

impl System {
    pub fn builder<'a, R, Q>() -> SystemBuilder<R, Q>
    where
        R: ResourceBundle + 'static,
        Q: QueryBundle + 'static,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn run(&mut self, world: &World) {
        self.inner.run(world);
    }
}

pub struct SystemBuilder<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    phantom_data: PhantomData<(R, Q)>,
}

impl<R, Q> SystemBuilder<R, Q>
where
    R: ResourceBundle + 'static,
    Q: QueryBundle + 'static,
{
    pub fn build<'a, F>(self, mut closure: F) -> System
    where
        R::Effectors: Fetch<'a>,
        F: FnMut(WorldProxy<'a>, <R::Effectors as Fetch<'a>>::Refs, Q::Effectors) + 'static,
    {
        let closure = Box::new(move |proxy, resources: R::Effectors, queries| {
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
                Box<dyn FnMut(WorldProxy<'a>, R::Effectors, Q::Effectors)>,
                Box<dyn FnMut(WorldProxy, R::Effectors, Q::Effectors)>,
            >(closure)
        };
        System {
            inner: Box::new(SystemBox::<R, Q> {
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
    let mut system = System::builder::<(&usize, &mut f32), (&usize, Option<&usize>)>().build(
        |world, (res1, mut res2), query| {
            *res2 += 1.0;
        },
    );
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 2.0);
    system.run(&world);
    assert_eq!(*world.fetch::<&f32>(), 3.0);
}

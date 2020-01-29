use hecs::World;
use resources::Resources;
use std::marker::PhantomData;

use crate::{
    query_bundle::{QueryBundle, QuerySingle},
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeSet, ModQueuePool, SystemBorrows, WorldFacade,
};

pub trait Runnable: Send {
    fn run(&mut self, facade: WorldFacade);

    fn write_borrows(&self, borrows: &mut SystemBorrows);

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet);
}

pub struct System {
    inner: Box<dyn Runnable>,
}

impl System {
    pub fn builder() -> SystemBuilder<(), (), ()> {
        SystemBuilder::new()
    }

    pub fn run(&mut self, world: &World, resources: &Resources, mod_queues: &ModQueuePool) {
        #[cfg(feature = "parallel")]
        self.inner
            .run(WorldFacade::new(world, resources, mod_queues, None));
        #[cfg(not(feature = "parallel"))]
        self.inner
            .run(WorldFacade::new(world, resources, mod_queues));
    }

    #[cfg(feature = "parallel")]
    pub(crate) fn inner(&self) -> &dyn Runnable {
        self.inner.as_ref()
    }
}

struct SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    phantom_data: PhantomData<(Comps, Res, Queries)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(WorldFacade, Res::Effectors, Queries::Effectors) + Send>,
}

impl<Comps, Res, Queries> Runnable for SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    fn run(&mut self, facade: WorldFacade) {
        (self.closure)(facade, Res::effectors(), Queries::effectors());
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        Comps::write_borrows(borrows);
        Res::write_borrows(borrows);
        Queries::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet) {
        Comps::write_archetypes(world, archetypes);
        Queries::write_archetypes(world, archetypes);
    }

    #[cfg(not(feature = "parallel"))]
    fn write_borrows(&self, _: &mut SystemBorrows) {}

    #[cfg(not(feature = "parallel"))]
    fn write_archetypes(&self, _: &World, _: &mut ArchetypeSet) {}
}

pub trait TupleAppend<T> {
    type Output;
}

impl<T0> TupleAppend<T0> for () {
    type Output = (T0,);
}

impl<T0, T1> TupleAppend<T1> for (T0,) {
    type Output = (T0, T1);
}

pub struct SystemBuilder<Comps, Res, Queries>
where
    Comps: QueryBundle + 'static,
    Res: ResourceBundle + 'static,
    Queries: QueryBundle + 'static,
{
    phantom_data: PhantomData<(Comps, Res, Queries)>,
}

impl SystemBuilder<(), (), ()> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }
}

impl<Comps, Queries> SystemBuilder<Comps, (), Queries>
where
    Comps: QueryBundle + 'static,
    Queries: QueryBundle + 'static,
{
    pub fn resources<Res>(self) -> SystemBuilder<Comps, Res, Queries>
    where
        Res: ResourceBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }
}

impl<Comps, Res, Queries> SystemBuilder<Comps, Res, Queries>
where
    Comps: QueryBundle + 'static,
    Res: ResourceBundle + 'static,
    Queries: QueryBundle + 'static,
{
    /*pub fn component<C>(self) -> SystemBuilder<Comps::Output, Res, Queries>
    where
        C: QueryUnit + QuerySingle,
        Comps: TupleAppend<C>,
        <Comps as TupleAppend<C>>::Output: QueryBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }*/

    pub fn query<Q>(self) -> SystemBuilder<Comps, Res, Queries::Output>
    where
        Q: QuerySingle,
        Queries: TupleAppend<Q>,
        <Queries as TupleAppend<Q>>::Output: QueryBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

    pub fn build<'a, F>(self, mut closure: F) -> System
    where
        Res::Effectors: Fetch<'a>,
        F: FnMut(WorldFacade<'a>, <Res::Effectors as Fetch<'a>>::Refs, Queries::Effectors)
            + Send
            + 'static,
    {
        let closure = Box::new(
            move |facade: WorldFacade<'a>, resource_effectors: Res::Effectors, queries| {
                let fetch = resource_effectors.fetch(&facade.resources);
                closure(facade, fetch, queries)
            },
        );
        let closure = unsafe {
            // FIXME this is a dirty hack for until
            //  https://github.com/rust-lang/rust/issues/62529 is fixed
            // I'm all but certain that this, within the context it's used in, is safe.
            // This transmutation forces the compiler to accept lifetime bounds it would've been
            // able to verify itself, if they were written as a HRTB.
            // Since HRTBs cause an ICE when used with closures in the way that's needed here
            // (see link above), I've opted for this workaround.
            std::mem::transmute::<
                Box<dyn FnMut(WorldFacade<'a>, Res::Effectors, Queries::Effectors) + Send>,
                Box<dyn FnMut(WorldFacade, Res::Effectors, Queries::Effectors) + Send>,
            >(closure)
        };
        let system_box = SystemBox::<Comps, Res, Queries> {
            phantom_data: PhantomData,
            closure,
        };
        System {
            inner: Box::new(system_box),
        }
    }
}

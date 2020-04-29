use hecs::World;
use resources::Resources;
use std::marker::PhantomData;

use crate::{
    query_bundle::{QueryBundle, QuerySingle},
    resource_bundle::{Fetch, ResourceBundle},
    ArchetypeAccess, ModQueuePool, SystemBorrows, SystemContext,
};

#[cfg(feature = "parallel")]
use crate::Scope;

pub trait Runnable {
    fn run(&mut self, context: SystemContext);

    fn write_borrows(&self, borrows: &mut SystemBorrows);

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeAccess);
}

pub struct System {
    inner: Box<dyn Runnable + Send>,
}

impl System {
    pub fn builder() -> SystemBuilder<(), ()> {
        SystemBuilder::new()
    }

    pub fn run(&mut self, world: &World, resources: &Resources, mod_queues: &ModQueuePool) {
        #[cfg(feature = "parallel")]
        self.inner
            .run(SystemContext::new(world, resources, mod_queues, None));
        #[cfg(not(feature = "parallel"))]
        self.inner
            .run(SystemContext::new(world, resources, mod_queues));
    }

    #[cfg(feature = "parallel")]
    pub fn run_with_scope(
        &mut self,
        world: &World,
        resources: &Resources,
        mod_queues: &ModQueuePool,
        scope: &Scope,
    ) {
        self.inner.run(SystemContext::new(
            world,
            resources,
            mod_queues,
            Some(scope),
        ));
    }

    #[cfg(feature = "parallel")]
    pub(crate) fn inner(&self) -> &dyn Runnable {
        self.inner.as_ref()
    }
}

struct SystemBox<Res, Queries>
where
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    phantom_data: PhantomData<(Res, Queries)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(SystemContext, Res::Effectors, Queries::Effectors) + Send>,
}

impl<Res, Queries> Runnable for SystemBox<Res, Queries>
where
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    fn run(&mut self, context: SystemContext) {
        (self.closure)(context, Res::effectors(), Queries::effectors());
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        Res::write_borrows(borrows);
        Queries::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeAccess) {
        Queries::write_archetypes(world, archetypes);
    }

    #[cfg(not(feature = "parallel"))]
    fn write_borrows(&self, _: &mut SystemBorrows) {}

    #[cfg(not(feature = "parallel"))]
    fn write_archetypes(&self, _: &World, _: &mut ArchetypeAccess) {}
}

pub trait TupleAppend<T> {
    type Output;
}

impl<T0> TupleAppend<T0> for () {
    type Output = (T0,);
}

pub struct SystemBuilder<Res, Queries>
where
    Res: ResourceBundle + 'static,
    Queries: QueryBundle + 'static,
{
    phantom_data: PhantomData<(Res, Queries)>,
}

impl SystemBuilder<(), ()> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }
}

impl<Queries> SystemBuilder<(), Queries>
where
    Queries: QueryBundle + 'static,
{
    pub fn resources<Res>(self) -> SystemBuilder<Res, Queries>
    where
        Res: ResourceBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }
}

impl<Res, Queries> SystemBuilder<Res, Queries>
where
    Res: ResourceBundle + 'static,
    Queries: QueryBundle + 'static,
{
    pub fn query<Q>(self) -> SystemBuilder<Res, Queries::Output>
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
        F: FnMut(SystemContext<'a>, <Res::Effectors as Fetch<'a>>::Refs, Queries::Effectors)
            + Send
            + 'static,
    {
        let closure = Box::new(
            move |context: SystemContext<'a>, resource_effectors: Res::Effectors, queries| {
                let fetch = resource_effectors.fetch(&context.resources);
                closure(context, fetch, queries)
            },
        );
        #[rustfmt::skip]
        let closure = unsafe {
            // FIXME this is a dirty hack for until
            //  https://github.com/rust-lang/rust/issues/62529 is fixed
            // I'm all but certain that this, within the context it's used in, is safe.
            // This transmutation forces the compiler to accept lifetime bounds it would've been
            // able to verify itself, if they were written as a HRTB.
            // Since HRTBs cause an ICE when used with closures in the way that's needed here
            // (see link above), I've opted for this workaround.
            std::mem::transmute::<
                Box<
                    dyn FnMut(SystemContext<'a>, Res::Effectors, Queries::Effectors)
                        + Send
                        + 'static,
                >,
                Box<
                    dyn FnMut(SystemContext, Res::Effectors, Queries::Effectors)
                        + Send
                        + 'static,
                >,
            >(closure)
        };
        let system_box = SystemBox::<Res, Queries> {
            phantom_data: PhantomData,
            closure,
        };
        System {
            inner: Box::new(system_box),
        }
    }
}

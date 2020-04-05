use hecs::World;
use resources::Resources;
use std::marker::PhantomData;

use crate::{
    query_bundle::{QueryBundle, QuerySingle, QueryUnit},
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
    pub fn builder() -> SystemBuilder<(), (), ()> {
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

struct SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    phantom_data: PhantomData<(Comps, Res, Queries)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(SystemContext, Res::Effectors, Queries::Effectors) + Send>,
}

impl<Comps, Res, Queries> Runnable for SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    fn run(&mut self, context: SystemContext) {
        (self.closure)(context, Res::effectors(), Queries::effectors());
    }

    #[cfg(feature = "parallel")]
    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        Comps::write_borrows(borrows);
        Res::write_borrows(borrows);
        Queries::write_borrows(borrows);
    }

    #[cfg(feature = "parallel")]
    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeAccess) {
        Comps::write_archetypes(world, archetypes);
        Queries::write_archetypes(world, archetypes);
    }

    #[cfg(not(feature = "parallel"))]
    fn write_borrows(&self, _: &mut SystemBorrows) {}

    #[cfg(not(feature = "parallel"))]
    fn write_archetypes(&self, _: &World, _: &mut ArchetypeAccess) {}
}

pub struct ThreadLocalSystem<'capture, T> {
    #[allow(clippy::type_complexity)]
    inner: Box<dyn for<'r> FnMut(SystemContext<'r>, &'r T) + 'capture>,
}

impl<'capture, T> ThreadLocalSystem<'capture, T> {
    pub fn run(
        &mut self,
        world: &World,
        resources: &Resources,
        mod_queues: &ModQueuePool,
        thread_local_resources: &mut T,
    ) {
        #[cfg(feature = "parallel")]
        (self.inner)(
            SystemContext::new(world, resources, mod_queues, None),
            thread_local_resources,
        );
        #[cfg(not(feature = "parallel"))]
        (self.inner)(
            SystemContext::new(world, resources, mod_queues),
            thread_local_resources,
        );
    }

    #[cfg(feature = "parallel")]
    pub fn run_with_scope(
        &mut self,
        world: &World,
        resources: &Resources,
        mod_queues: &ModQueuePool,
        scope: &Scope,
        thread_local_resources: &mut T,
    ) {
        (self.inner)(
            SystemContext::new(world, resources, mod_queues, Some(scope)),
            thread_local_resources,
        );
    }
}

pub trait TupleAppend<T> {
    type Output;
}

impl<T0> TupleAppend<T0> for () {
    type Output = (T0,);
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
    pub fn component<C>(self) -> SystemBuilder<Comps::Output, Res, Queries>
    where
        C: QueryUnit + QuerySingle,
        Comps: TupleAppend<C>,
        <Comps as TupleAppend<C>>::Output: QueryBundle,
    {
        SystemBuilder {
            phantom_data: PhantomData,
        }
    }

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
        let system_box = SystemBox::<Comps, Res, Queries> {
            phantom_data: PhantomData,
            closure,
        };
        System {
            inner: Box::new(system_box),
        }
    }

    pub fn build_thread_local<'capture, 'run, F, T>(
        self,
        mut closure: F,
    ) -> ThreadLocalSystem<'capture, T>
    where
        Res::Effectors: Fetch<'run>,
        F: FnMut(
                SystemContext<'run>,
                <Res::Effectors as Fetch<'run>>::Refs,
                Queries::Effectors,
                &'run mut T,
            ) + 'capture,
        T: 'run,
    {
        let closure: Box<dyn FnMut(SystemContext<'run>, &'run mut T) + 'capture> = Box::new(
            move |context: SystemContext<'run>, thread_local_resources| {
                let fetch = Res::effectors().fetch(&context.resources);
                closure(context, fetch, Queries::effectors(), thread_local_resources)
            },
        );
        ThreadLocalSystem {
            inner: unsafe {
                // See note in `build()`.
                std::mem::transmute(closure)
            },
        }
    }
}

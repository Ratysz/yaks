use fxhash::FxHasher64;
use std::{any::TypeId, collections::HashSet, hash::BuildHasherDefault, marker::PhantomData};

use crate::{
    query_bundle::{QueryBundle, QuerySingle},
    resource_bundle::{Fetch, ResourceBundle},
    World,
};

pub type TypeSet = HashSet<TypeId, BuildHasherDefault<FxHasher64>>;
pub type ArchetypeSet = HashSet<u32, BuildHasherDefault<FxHasher64>>;

#[derive(Default)]
pub struct SystemBorrows {
    pub resources_immutable: TypeSet,
    pub resources_mutable: TypeSet,
    pub components_immutable: TypeSet,
    pub components_mutable: TypeSet,
}

pub(crate) trait SystemTrait {
    fn run(&mut self, world: &World);

    fn write_borrows(&self, borrows: &mut SystemBorrows);

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet);
}

pub struct System {
    inner: Box<dyn SystemTrait>,
}

impl System {
    pub fn builder() -> SystemBuilder<(), (), ()> {
        SystemBuilder::new()
    }

    pub fn run_and_flush(&mut self, world: &mut World) {
        self.run(world);
        world.flush_mod_queues();
    }

    pub fn run(&mut self, world: &World) {
        self.inner.run(world);
    }

    /*pub(crate) fn inner(&self) -> &dyn SystemTrait {
        self.inner.as_ref()
    }*/
}

struct SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    phantom_data: PhantomData<(Comps, Res, Queries)>,
    #[allow(clippy::type_complexity)]
    closure: Box<dyn FnMut(&World, Res::Effectors, Queries::Effectors)>,
}

impl<Comps, Res, Queries> SystemTrait for SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World) {
        (self.closure)(world, Res::effectors(), Queries::effectors());
    }

    fn write_borrows(&self, borrows: &mut SystemBorrows) {
        Comps::write_borrows(borrows);
        Res::write_borrows(borrows);
        Queries::write_borrows(borrows);
    }

    fn write_archetypes(&self, world: &World, archetypes: &mut ArchetypeSet) {
        Comps::write_archetypes(world, archetypes);
        Queries::write_archetypes(world, archetypes);
    }
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
        F: FnMut(&'a World, <Res::Effectors as Fetch<'a>>::Refs, Queries::Effectors) + 'static,
    {
        let closure = Box::new(
            move |world: &'a World, resources: Res::Effectors, queries| {
                closure(world, resources.fetch(world), queries)
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
                Box<dyn FnMut(&'a World, Res::Effectors, Queries::Effectors)>,
                Box<dyn FnMut(&World, Res::Effectors, Queries::Effectors)>,
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

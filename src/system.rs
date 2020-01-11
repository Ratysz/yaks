use fxhash::FxHasher64;
use std::{
    any::TypeId, collections::HashSet, fmt::Debug, hash::BuildHasherDefault, marker::PhantomData,
};

use crate::{
    modification_queue::ModificationQueue,
    query_bundle::{QueryBundle, QuerySingle, QueryUnit},
    resource_bundle::{Fetch, ResourceBundle},
    World, WorldProxy,
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

trait SystemTrait {
    fn run(&mut self, world: &World) -> ModificationQueue;

    fn debug_id(&self) -> &str;

    fn borrows(&self) -> &SystemBorrows;

    fn update_archetypes(&mut self, world: &World);

    fn archetypes(&self) -> &ArchetypeSet;
}

pub struct System {
    inner: Box<dyn SystemTrait>,
}

impl System {
    pub fn builder<T>(debug_id: T) -> SystemBuilder<(), (), ()>
    where
        T: Debug,
    {
        SystemBuilder::new(debug_id)
    }

    pub fn run(&mut self, world: &mut World) {
        world.apply_all(self.run_with_deferred_modification(world));
    }

    pub fn run_with_deferred_modification(&mut self, world: &World) -> ModificationQueue {
        self.update_archetypes(world);
        self.run_without_updating_archetypes(world)
    }

    pub(crate) fn run_without_updating_archetypes(&mut self, world: &World) -> ModificationQueue {
        self.inner.run(world)
    }

    pub(crate) fn borrows(&self) -> &SystemBorrows {
        self.inner.borrows()
    }

    pub(crate) fn update_archetypes(&mut self, world: &World) {
        self.inner.update_archetypes(world);
    }

    pub(crate) fn archetypes(&self) -> &ArchetypeSet {
        &self.inner.archetypes()
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
    closure: Box<dyn FnMut(&mut WorldProxy, Res::Effectors, Queries::Effectors)>,
    debug_id: String,
    borrows: SystemBorrows,
    archetypes: ArchetypeSet,
}

impl<Comps, Res, Queries> SystemTrait for SystemBox<Comps, Res, Queries>
where
    Comps: QueryBundle,
    Res: ResourceBundle,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World) -> ModificationQueue {
        let mut queue = world.modification_queue();
        (self.closure)(
            &mut WorldProxy::new(
                &world,
                &mut queue,
                &self.debug_id,
                &self.borrows,
                &self.archetypes,
            ),
            Res::effectors(),
            Queries::effectors(),
        );
        queue
    }

    fn debug_id(&self) -> &str {
        &self.debug_id
    }

    fn borrows(&self) -> &SystemBorrows {
        &self.borrows
    }

    fn update_archetypes(&mut self, world: &World) {
        Comps::write_archetypes(world, &mut self.archetypes);
        Queries::write_archetypes(world, &mut self.archetypes);
    }

    fn archetypes(&self) -> &ArchetypeSet {
        &self.archetypes
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
    debug_id: String,
}

impl SystemBuilder<(), (), ()> {
    pub fn new<T>(debug_id: T) -> Self
    where
        T: Debug,
    {
        SystemBuilder {
            phantom_data: PhantomData,
            debug_id: format!("{:?}", debug_id),
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
            debug_id: self.debug_id,
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
            debug_id: self.debug_id,
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
            debug_id: self.debug_id,
        }
    }

    pub fn build<'a, F>(self, mut closure: F) -> System
    where
        Res::Effectors: Fetch<'a>,
        F: FnMut(&mut WorldProxy<'a>, <Res::Effectors as Fetch<'a>>::Refs, Queries::Effectors)
            + 'static,
    {
        let closure = Box::new(
            move |proxy: &mut WorldProxy<'a>, resources: Res::Effectors, queries| {
                closure(proxy, resources.fetch(proxy.world), queries)
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
                Box<dyn FnMut(&mut WorldProxy<'a>, Res::Effectors, Queries::Effectors)>,
                Box<dyn FnMut(&mut WorldProxy, Res::Effectors, Queries::Effectors)>,
            >(closure)
        };

        let mut borrows = SystemBorrows::default();
        Comps::write_borrows(&mut borrows);
        Res::write_borrows(&mut borrows);
        Queries::write_borrows(&mut borrows);

        let system_box = SystemBox::<Comps, Res, Queries> {
            phantom_data: PhantomData,
            closure,
            debug_id: self.debug_id,
            borrows,
            archetypes: ArchetypeSet::default(),
        };
        System {
            inner: Box::new(system_box),
        }
    }
}

use hecs::{Component, Entity, EntityRef, MissingComponent, Query, Ref, RefMut, World};
use std::marker::PhantomData;

use crate::query_bundle::QuerySingle;

pub struct Immutable;

pub struct Mutable;

pub trait Mutability: Send + Sync {}

impl Mutability for Immutable {}

impl Mutability for Mutable {}

pub struct Mandatory;

pub struct Optional;

pub trait Optionality: Send + Send {}

impl Optionality for Mandatory {}

impl Optionality for Optional {}

pub struct ComponentEffector<M, O, C>
where
    M: Mutability,
    O: Optionality,
    C: Component + Send + Sync,
{
    phantom_data: PhantomData<(M, O, C)>,
}

impl<M, O, C> ComponentEffector<M, O, C>
where
    M: Mutability,
    O: Optionality,
    C: Component + Send + Sync,
{
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

pub trait Fetch<'a> {
    type Refs;

    fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs;
}

impl<'a> Fetch<'a> for () {
    type Refs = ();

    fn fetch(&self, _: EntityRef<'a>) -> Self::Refs {}
}

impl<'a, C> Fetch<'a> for ComponentEffector<Immutable, Mandatory, C>
where
    C: Component,
{
    type Refs = Ref<'a, C>;

    fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs {
        entity_ref
            .get::<C>()
            .unwrap_or_else(|| panic!("cannot fetch: {}", MissingComponent::new::<C>()))
    }
}

impl<'a, C> Fetch<'a> for ComponentEffector<Mutable, Mandatory, C>
where
    C: Component,
{
    type Refs = RefMut<'a, C>;

    fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs {
        entity_ref
            .get_mut::<C>()
            .unwrap_or_else(|| panic!("cannot fetch: {}", MissingComponent::new::<C>()))
    }
}

impl<'a, C> Fetch<'a> for ComponentEffector<Immutable, Optional, C>
where
    C: Component,
{
    type Refs = Option<Ref<'a, C>>;

    fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs {
        entity_ref.get::<C>()
    }
}

impl<'a, C> Fetch<'a> for ComponentEffector<Mutable, Optional, C>
where
    C: Component,
{
    type Refs = Option<RefMut<'a, C>>;

    fn fetch(&self, entity_ref: EntityRef<'a>) -> Self::Refs {
        entity_ref.get_mut::<C>()
    }
}

pub trait FetchComponents {
    fn fetch<Q>(&self, entity: Entity) -> <Q::ComponentEffectors as Fetch>::Refs
    where
        Q: Query + QuerySingle,
        for<'a> Q::ComponentEffectors: Fetch<'a>;
}

impl FetchComponents for World {
    fn fetch<Q>(&self, entity: Entity) -> <Q::ComponentEffectors as Fetch>::Refs
    where
        Q: Query + QuerySingle,
        for<'a> Q::ComponentEffectors: Fetch<'a>,
    {
        Q::component_effectors().fetch(
            self.entity(entity)
                .unwrap_or_else(|error| panic!("cannot get entity {:?}: {}", entity, error)),
        )
    }
}

use crate::{
    ArchetypeSet, BorrowSet, BorrowTypeSet, ContainsMut, ContainsRef, Query, QueryExt,
    ResourceTuple,
};

pub trait SystemArgument<'a, ExecutorResources, Marker>
where
    ExecutorResources: ResourceTuple,
{
    fn fetch(world: &'a hecs::World, resources: &'a ExecutorResources::Wrapped) -> Self;

    unsafe fn release(_: &'a hecs::World, _: &'a ExecutorResources::Wrapped) {}

    fn set_resource_bits(_: &mut BorrowSet) {}

    fn insert_component_types(_: &mut BorrowTypeSet) {}

    fn set_archetype_bits(_: &hecs::World, _: &mut ArchetypeSet) {}
}

impl<'a, ExecutorResources, QueryInner> SystemArgument<'a, ExecutorResources, ()>
    for Query<'a, QueryInner>
where
    ExecutorResources: ResourceTuple,
    QueryInner: QueryExt,
{
    fn fetch(world: &'a hecs::World, _: &'a ExecutorResources::Wrapped) -> Self {
        Query::new(world)
    }

    fn insert_component_types(component_type_set: &mut BorrowTypeSet) {
        QueryInner::insert_component_types(component_type_set);
    }

    fn set_archetype_bits(world: &hecs::World, archetype_set: &mut ArchetypeSet) {
        archetype_set.set_bits_for_query::<QueryInner>(world);
    }
}

impl<'a, ExecutorResources, Marker, Resource> SystemArgument<'a, ExecutorResources, Marker>
    for &'a Resource
where
    ExecutorResources: ResourceTuple,
    ExecutorResources::Wrapped: ContainsRef<Resource, Marker> + 'a,
    Resource: 'a,
{
    fn fetch(_: &'a hecs::World, resources: &'a ExecutorResources::Wrapped) -> Self {
        ExecutorResources::Wrapped::borrow_ref(resources)
    }

    unsafe fn release(_: &'a hecs::World, resources: &'a ExecutorResources::Wrapped) {
        ExecutorResources::Wrapped::release_ref(resources)
    }

    fn set_resource_bits(resource_set: &mut BorrowSet) {
        ExecutorResources::Wrapped::set_resource_bit(&mut resource_set.immutable)
    }
}

impl<'a, ExecutorResources, Marker, Resource> SystemArgument<'a, ExecutorResources, Marker>
    for &'a mut Resource
where
    ExecutorResources: ResourceTuple,
    ExecutorResources::Wrapped: ContainsMut<Resource, Marker> + 'a,
    Resource: 'a,
{
    fn fetch(_: &'a hecs::World, resources: &'a ExecutorResources::Wrapped) -> Self {
        ExecutorResources::Wrapped::borrow_mut(resources)
    }

    unsafe fn release(_: &'a hecs::World, resources: &'a ExecutorResources::Wrapped) {
        ExecutorResources::Wrapped::release_mut(resources)
    }

    fn set_resource_bits(resource_set: &mut BorrowSet) {
        ExecutorResources::Wrapped::set_resource_bit(&mut resource_set.mutable)
    }
}

use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[cfg(feature = "parallel")]
use hecs::World;

use crate::{
    DerefTuple, Executor, Fetch, QueryBundle, ResourceTuple, SystemClosure, SystemContext,
    WrappedResources,
};

#[cfg(feature = "parallel")]
use crate::{ArchetypeSet, ComponentTypeSet, ResourceSet, TypeSet};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SystemId(usize);

pub struct System<'closure, Resources>
where
    Resources: ResourceTuple + 'closure,
{
    pub closure: Box<SystemClosure<'closure, Resources::Cells>>,
    pub dependencies: Vec<SystemId>,
    #[cfg(feature = "parallel")]
    pub resource_set: ResourceSet,
    #[cfg(feature = "parallel")]
    pub component_type_set: ComponentTypeSet,
    #[cfg(feature = "parallel")]
    pub archetype_writer: Box<dyn Fn(&World, &mut ArchetypeSet) + Send>,
}

pub struct ExecutorBuilder<'closures, Resources, Handle = DummyHandle>
where
    Resources: ResourceTuple,
{
    pub(crate) systems: HashMap<SystemId, System<'closures, Resources>>,
    pub(crate) handles: HashMap<Handle, SystemId>,
    #[cfg(feature = "parallel")]
    pub(crate) all_component_types: TypeSet,
}

impl<'closures, Resources> ExecutorBuilder<'closures, Resources, DummyHandle>
where
    Resources: ResourceTuple,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            handles: HashMap::with_capacity(0),
            #[cfg(feature = "parallel")]
            all_component_types: TypeSet::new(),
        }
    }
}

impl<'closures, Resources, Handle> ExecutorBuilder<'closures, Resources, Handle>
where
    Resources: ResourceTuple,
    Handle: Eq + Hash,
{
    fn box_system<'a, Closure, ResourceRefs, Queries, Markers>(
        mut closure: Closure,
    ) -> System<'closures, Resources>
    where
        Resources::Cells: 'a,
        Closure: FnMut(SystemContext<'a>, ResourceRefs, Queries) + Send + Sync + 'closures,
        ResourceRefs: Fetch<'a, WrappedResources<'a, Resources::Cells>, Markers> + 'a,
        Queries: QueryBundle,
    {
        let closure = Box::new(
            move |context: SystemContext<'a>, resources: &'a WrappedResources<Resources::Cells>| {
                let mut fetched = ResourceRefs::fetch(resources);
                closure(context, unsafe { fetched.deref() }, Queries::markers());
                ResourceRefs::release(resources, fetched);
            },
        );
        let closure = unsafe {
            std::mem::transmute::<
                Box<dyn FnMut(_, &'a _) + Send + Sync + 'closures>,
                Box<
                    dyn FnMut(SystemContext, &WrappedResources<Resources::Cells>)
                        + Send
                        + Sync
                        + 'closures,
                >,
            >(closure)
        };
        #[cfg(feature = "parallel")]
        {
            let mut resource_set = ResourceSet::with_capacity(Resources::LENGTH);
            ResourceRefs::set_resource_bits(&mut resource_set);
            let mut component_type_set =
                ComponentTypeSet::with_capacity(Queries::COMPONENT_TYPE_SET_LENGTH);
            Queries::insert_component_types(&mut component_type_set);
            let archetype_writer = Box::new(|world: &World, archetype_set: &mut ArchetypeSet| {
                Queries::set_archetype_bits(world, archetype_set)
            });
            System {
                closure,
                dependencies: vec![],
                resource_set,
                component_type_set,
                archetype_writer,
            }
        }
        #[cfg(not(feature = "parallel"))]
        System {
            closure,
            dependencies: vec![],
        }
    }

    pub fn system<'a, Closure, ResourceRefs, Queries, Markers>(mut self, closure: Closure) -> Self
    where
        Resources::Cells: 'a,
        Closure: FnMut(SystemContext<'a>, ResourceRefs, Queries) + Send + Sync + 'closures,
        ResourceRefs: Fetch<'a, WrappedResources<'a, Resources::Cells>, Markers> + 'a,
        Queries: QueryBundle,
    {
        let id = SystemId(self.systems.len());
        let system = Self::box_system::<'a, Closure, ResourceRefs, Queries, Markers>(closure);
        #[cfg(feature = "parallel")]
        {
            self.all_component_types
                .extend(&system.component_type_set.immutable);
            self.all_component_types
                .extend(&system.component_type_set.mutable);
        }
        self.systems.insert(id, system);
        self
    }

    pub fn system_with_handle<'a, Closure, ResourceRefs, Queries, Markers, NewHandle>(
        mut self,
        closure: Closure,
        handle: NewHandle,
    ) -> ExecutorBuilder<'closures, Resources, NewHandle>
    where
        Resources::Cells: 'a,
        Closure: FnMut(SystemContext<'a>, ResourceRefs, Queries) + Send + Sync + 'closures,
        ResourceRefs: Fetch<'a, WrappedResources<'a, Resources::Cells>, Markers> + 'a,
        Queries: QueryBundle,
        NewHandle: HandleConversion<Handle> + Debug,
    {
        let mut handles = NewHandle::convert_hash_map(self.handles);
        if handles.contains_key(&handle) {
            panic!("system {:?} already exists", handle);
        }
        let id = SystemId(self.systems.len());
        let system = Self::box_system::<'a, Closure, ResourceRefs, Queries, Markers>(closure);
        #[cfg(feature = "parallel")]
        {
            self.all_component_types
                .extend(&system.component_type_set.immutable);
            self.all_component_types
                .extend(&system.component_type_set.mutable);
        }
        self.systems.insert(id, system);
        handles.insert(handle, id);
        ExecutorBuilder {
            systems: self.systems,
            handles,
            #[cfg(feature = "parallel")]
            all_component_types: self.all_component_types,
        }
    }

    pub fn system_with_deps<'a, Closure, ResourceRefs, Queries, Markers>(
        mut self,
        closure: Closure,
        dependencies: Vec<Handle>,
    ) -> Self
    where
        Resources::Cells: 'a,
        Closure: FnMut(SystemContext<'a>, ResourceRefs, Queries) + Send + Sync + 'closures,
        ResourceRefs: Fetch<'a, WrappedResources<'a, Resources::Cells>, Markers> + 'a,
        Queries: QueryBundle,
        Handle: Eq + Hash + Debug,
    {
        let id = SystemId(self.systems.len());
        let mut system = Self::box_system::<'a, Closure, ResourceRefs, Queries, Markers>(closure);
        #[cfg(feature = "parallel")]
        {
            self.all_component_types
                .extend(&system.component_type_set.immutable);
            self.all_component_types
                .extend(&system.component_type_set.mutable);
        }
        system
            .dependencies
            .extend(dependencies.iter().map(|dep_handle| {
                *self.handles.get(dep_handle).unwrap_or_else(|| {
                    panic!(
                    "could not resolve dependencies of a handle-less system: no system {:?} found",
                    dep_handle
                )
                })
            }));
        self.systems.insert(id, system);
        self
    }

    pub fn system_with_handle_and_deps<'a, Closure, ResourceRefs, Queries, Markers>(
        mut self,
        closure: Closure,
        handle: Handle,
        dependencies: Vec<Handle>,
    ) -> Self
    where
        Resources::Cells: 'a,
        Closure: FnMut(SystemContext<'a>, ResourceRefs, Queries) + Send + Sync + 'closures,
        ResourceRefs: Fetch<'a, WrappedResources<'a, Resources::Cells>, Markers> + 'a,
        Queries: QueryBundle,
        Handle: Eq + Hash + Debug,
    {
        if self.handles.contains_key(&handle) {
            panic!("system {:?} already exists", handle);
        }
        if dependencies.contains(&handle) {
            panic!("system {:?} depends on itself", handle);
        }
        let id = SystemId(self.systems.len());
        let mut system = Self::box_system::<'a, Closure, ResourceRefs, Queries, Markers>(closure);
        #[cfg(feature = "parallel")]
        {
            self.all_component_types
                .extend(&system.component_type_set.immutable);
            self.all_component_types
                .extend(&system.component_type_set.mutable);
        }
        system
            .dependencies
            .extend(dependencies.iter().map(|dep_handle| {
                *self.handles.get(dep_handle).unwrap_or_else(|| {
                    panic!(
                        "could not resolve dependencies of system {:?}: no system {:?} found",
                        handle, dep_handle
                    )
                })
            }));
        self.systems.insert(id, system);
        self.handles.insert(handle, id);
        self
    }

    pub fn build(self) -> Executor<'closures, Resources> {
        Executor::build(self)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct DummyHandle;

pub trait HandleConversion<T>: Sized + Eq + Hash {
    fn convert_hash_map(map: HashMap<T, SystemId>) -> HashMap<Self, SystemId>;
}

impl<T> HandleConversion<DummyHandle> for T
where
    T: Debug + Eq + Hash,
{
    fn convert_hash_map(_: HashMap<DummyHandle, SystemId>) -> HashMap<Self, SystemId> {
        HashMap::new()
    }
}

impl<T> HandleConversion<T> for T
where
    T: Debug + Eq + Hash,
{
    fn convert_hash_map(map: HashMap<T, SystemId>) -> HashMap<Self, SystemId> {
        map
    }
}

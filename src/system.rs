use crate::{Fetch, Query, ResourceTuple, SystemId};

#[cfg(feature = "parallel")]
use crate::{ArchetypeSet, BorrowSet, BorrowTypeSet, QueryExt};

#[cfg(not(feature = "parallel"))]
use hecs::Query as QueryExt;

pub type SystemClosure<'closure, Cells> = dyn FnMut(&hecs::World, &Cells) + Send + Sync + 'closure;

/// Container for system and metadata, parsed for use in a specific executor.
pub struct System<'closure, ExecutorResources>
where
    ExecutorResources: ResourceTuple + 'closure,
{
    pub closure: Box<SystemClosure<'closure, ExecutorResources::Wrapped>>,
    pub dependencies: Vec<SystemId>,
    #[cfg(feature = "parallel")]
    pub resource_set: BorrowSet,
    #[cfg(feature = "parallel")]
    pub component_type_set: BorrowTypeSet,
    #[cfg(feature = "parallel")]
    pub archetype_writer: Box<dyn Fn(&hecs::World, &mut ArchetypeSet) + Send>,
}

pub trait IntoSystem<'closure, ExecutorResources, Markers, Resources, Queries>
where
    ExecutorResources: ResourceTuple + 'closure,
{
    fn into_system(self) -> System<'closure, ExecutorResources>;
}

macro_rules! impl_into_system {
    ($resource:ident, ($($query:ident,)*)) => {};
    ((), ($($query:ident,)*)) => {
        impl<'a, 'closure, Closure, ExecutorResources, $($query,)*>
            IntoSystem<'closure, ExecutorResources, (), (), ($($query,)*)> for Closure
        where
            Closure: FnMut($(Query<$query>,)*) + Send + Sync + 'closure,
            ExecutorResources: ResourceTuple + 'closure,
            ExecutorResources::Wrapped: 'a,
            $($query: QueryExt,)*
        {
            #[allow(non_snake_case, unused_variables, unused_mut)]
            fn into_system(mut self) -> System<'closure, ExecutorResources> {
                let closure = Box::new(
                    move |world: &'a hecs::World, _: &'a ExecutorResources::Wrapped| {
                        self($(Query::<$query>::new(world),)*);
                    }
                );
                let closure = unsafe {
                    std::mem::transmute::<
                        Box<dyn FnMut(&'a _, &'a _) + Send + Sync + 'closure>,
                        Box<dyn FnMut(&_, &_) + Send + Sync + 'closure>,
                    >(closure)
                };
                #[cfg(feature = "parallel")]
                {
                    let resource_set = BorrowSet::with_capacity(ExecutorResources::LENGTH);
                    let mut component_type_set = BorrowTypeSet::new();
                    $($query::insert_component_types(&mut component_type_set);)*
                    let archetype_writer =
                        Box::new(|world: &hecs::World, archetype_set: &mut ArchetypeSet| {
                            $($query::set_archetype_bits(world, archetype_set);)*
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
        }
    };
    (($($resource:ident,)*), ($($query:ident,)*)) => {
        impl<'a, 'closure, Closure, ExecutorResources, Markers, $($resource,)* $($query,)*>
            IntoSystem<'closure, ExecutorResources, Markers, ($($resource,)*), ($($query,)*)>
            for Closure
        where
            Closure: FnMut($($resource,)* $(Query<$query>,)*) + Send + Sync + 'closure,
            ExecutorResources: ResourceTuple + 'closure,
            ExecutorResources::Wrapped: 'a,
            ($($resource,)*): Fetch<&'a ExecutorResources::Wrapped, Markers> + 'a,
            $($query: QueryExt,)*
        {
            #[allow(non_snake_case, unused_variables, unused_mut)]
            fn into_system(mut self) -> System<'closure, ExecutorResources> {
                let closure = Box::new(
                    move |world: &'a hecs::World, resources: &'a ExecutorResources::Wrapped| {
                        let ($($resource,)*) = <($($resource,)*)>::fetch(resources);
                        self($($resource,)* $(Query::<$query>::new(world),)*);
                        unsafe { <($($resource,)*)>::release(resources) };
                    }
                );
                let closure = unsafe {
                    std::mem::transmute::<
                        Box<dyn FnMut(&'a _, &'a _) + Send + Sync + 'closure>,
                        Box<dyn FnMut(&_, &_) + Send + Sync + 'closure>,
                    >(closure)
                };
                #[cfg(feature = "parallel")]
                {
                    let mut resource_set = BorrowSet::with_capacity(ExecutorResources::LENGTH);
                    <($($resource,)*)>::set_resource_bits(&mut resource_set);
                    let mut component_type_set = BorrowTypeSet::new();
                    $($query::insert_component_types(&mut component_type_set);)*
                    let archetype_writer =
                        Box::new(|world: &hecs::World, archetype_set: &mut ArchetypeSet| {
                            $($query::set_archetype_bits(world, archetype_set);)*
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
        }
    };
}

impl_for_res_and_query_tuples!(impl_into_system);

#[test]
fn smoke_test() {
    use crate::{
        resource::{AtomicBorrow, WrappableSingle},
        Mut, Ref,
    };
    let world = hecs::World::new();
    let mut counter = 0i32;
    let increment = 3usize;
    let mut borrows = (AtomicBorrow::new(), AtomicBorrow::new());
    let wrapped = (
        wrap_helper!(mut counter, i32, borrows.0),
        wrap_helper!(increment, usize, borrows.1),
    );

    fn increment_system(value: &mut i32) {
        *value += 1;
    }
    let mut boxed: System<(Mut<i32>, Ref<usize>)> = increment_system.into_system();
    (boxed.closure)(&world, &wrapped);
    assert_eq!(counter, 1);

    fn sum_system(a: &mut i32, b: &usize) {
        *a += *b as i32;
    }
    let mut boxed: System<(Mut<i32>, Ref<usize>)> = sum_system.into_system();
    (boxed.closure)(&world, &wrapped);
    assert_eq!(counter, 4);
    (boxed.closure)(&world, &wrapped);
    assert_eq!(counter, 7);
}

use hecs::World;
use resources::{Resource, Resources};

use crate::{
    AtomicBorrow, Executor, Mut, QueryBundle, Ref, RefExtractor, ResourceMutCell, ResourceRefCell,
    ResourceTuple, System, SystemContext,
};

// TODO sprinkle this in doc examples

pub trait WrappableSingle<'a> {
    type Fetched;
    type Wrapped;

    fn fetch(resources: &'a Resources) -> Self::Fetched;

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped;
}

impl<'a, R0> WrappableSingle<'a> for Ref<R0>
where
    R0: Resource,
{
    type Fetched = resources::Ref<'a, R0>;
    type Wrapped = ResourceRefCell<R0>;

    fn fetch(resources: &'a Resources) -> Self::Fetched {
        resources.get().unwrap_or_else(|error| panic!("{}", error))
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceRefCell::new(fetched, borrow)
    }
}

impl<'a, R0> WrappableSingle<'a> for Mut<R0>
where
    R0: Resource,
{
    type Fetched = resources::RefMut<'a, R0>;
    type Wrapped = ResourceMutCell<R0>;

    fn fetch(resources: &'a Resources) -> Self::Fetched {
        resources
            .get_mut()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn wrap(fetched: &mut Self::Fetched, borrow: &mut AtomicBorrow) -> Self::Wrapped {
        ResourceMutCell::new(fetched, borrow)
    }
}

pub trait Wrappable<'a> {
    type Fetched;
    type Wrapped;
    type BorrowTuple;

    fn fetch(resources: &'a Resources) -> Self::Fetched;

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped;
}

impl<'a, R0> Wrappable<'a> for Ref<R0>
where
    R0: Resource,
{
    type Fetched = resources::Ref<'a, R0>;
    type Wrapped = (ResourceRefCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn fetch(resources: &'a Resources) -> Self::Fetched {
        resources.get().unwrap_or_else(|error| panic!("{}", error))
    }

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceRefCell::new(fetched, &mut borrows.0),)
    }
}

impl<'a, R0> Wrappable<'a> for Mut<R0>
where
    R0: Resource,
{
    type Fetched = resources::RefMut<'a, R0>;
    type Wrapped = (ResourceMutCell<R0>,);
    type BorrowTuple = (AtomicBorrow,);

    fn fetch(resources: &'a Resources) -> Self::Fetched {
        resources
            .get_mut()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (ResourceMutCell::new(fetched, &mut borrows.0),)
    }
}

impl<'a> Wrappable<'a> for () {
    type Fetched = ();
    type Wrapped = ();
    type BorrowTuple = ();

    fn fetch(_: &'a Resources) -> Self::Fetched {}

    fn wrap(_: &mut Self::Fetched, _: &mut Self::BorrowTuple) -> Self::Wrapped {}
}

impl<'a, F0> Wrappable<'a> for (F0,)
where
    F0: WrappableSingle<'a> + 'a,
{
    type Fetched = (F0::Fetched,);
    type Wrapped = (F0::Wrapped,);
    type BorrowTuple = (AtomicBorrow,);

    fn fetch(resources: &'a Resources) -> Self::Fetched {
        (F0::fetch(resources),)
    }

    fn wrap(fetched: &mut Self::Fetched, borrows: &mut Self::BorrowTuple) -> Self::Wrapped {
        (F0::wrap(&mut fetched.0, &mut borrows.0),)
    }
}

macro_rules! swap_to_atomic_borrow {
    ($anything:tt) => {
        AtomicBorrow
    };
}

macro_rules! impl_wrappable {
    ($($letter:ident),*) => {
        paste::item! {
            impl<'a, $($letter),*> Wrappable<'a> for ($($letter,)*)
            where
                $($letter: WrappableSingle<'a> + 'a,)*
            {
                type Fetched = ($($letter::Fetched,)*);
                type Wrapped = ($($letter::Wrapped,)*);
                type BorrowTuple = ($(swap_to_atomic_borrow!($letter),)*);

                fn fetch(resources: &'a Resources) -> Self::Fetched {
                    ($($letter::fetch(resources),)*)
                }

                #[allow(non_snake_case)]
                fn wrap(
                    fetched: &mut Self::Fetched,
                    borrows: &mut Self::BorrowTuple
                ) -> Self::Wrapped {
                    let ($([<S $letter>],)*) = fetched;
                    let ($([<B $letter>],)*) = borrows;
                    ($($letter::wrap([<S $letter>], [<B $letter>]),)*)
                }
            }
        }
    }
}

impl_for_tuples!(impl_wrappable);

impl<F> RefExtractor<&Resources, Resources> for F
where
    Self: ResourceTuple,
    for<'a> Self: Wrappable<'a, Wrapped = Self::Wrapped, BorrowTuple = Self::BorrowTuple>,
{
    fn extract_and_run(executor: &mut Executor<Self>, world: &World, resources: &Resources) {
        let mut fetched = F::fetch(resources);
        let wrapped = F::wrap(&mut fetched, &mut executor.borrows);
        executor.inner.run(world, wrapped);
    }
}

pub trait FetchForSystem<'a> {
    type Wrapped;

    fn fetch(resources: &'a Resources) -> Self::Wrapped;

    fn deref(wrapped: &mut Self::Wrapped) -> Self;
}

impl<'a, R0> FetchForSystem<'a> for &'_ R0
where
    R0: Resource,
{
    type Wrapped = resources::Ref<'a, R0>;

    fn fetch(resources: &'a Resources) -> Self::Wrapped {
        resources.get().unwrap_or_else(|error| panic!("{}", error))
    }

    fn deref(wrapped: &mut Self::Wrapped) -> Self {
        unsafe { std::mem::transmute(&**wrapped) }
    }
}

impl<'a, R0> FetchForSystem<'a> for &'_ mut R0
where
    R0: Resource,
{
    type Wrapped = resources::RefMut<'a, R0>;

    fn fetch(resources: &'a Resources) -> Self::Wrapped {
        resources
            .get_mut()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn deref(wrapped: &mut Self::Wrapped) -> Self {
        unsafe { std::mem::transmute(&mut **wrapped) }
    }
}

impl<'a, 'closure, Closure, Queries> System<'closure, (), Queries, &'a Resources, Resources>
    for Closure
where
    Closure: FnMut(SystemContext, (), Queries) + 'closure,
    Closure: System<'closure, (), Queries, (), ()>,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World, _: &'a Resources) {
        self.run(world, ());
    }
}

impl<'a, 'closure, Closure, A, Queries> System<'closure, A, Queries, &'a Resources, Resources>
    for Closure
where
    Closure: FnMut(SystemContext, A, Queries) + 'closure,
    Closure: System<'closure, A, Queries, A, ()>,
    for<'r> A: FetchForSystem<'r>,
    Queries: QueryBundle,
{
    fn run(&mut self, world: &World, resources: &'a Resources) {
        let mut a = A::fetch(resources);
        self.run(world, A::deref(&mut a));
    }
}

macro_rules! impl_system {
    ($($letter:ident),*) => {
        impl<'a, 'closure, Closure, $($letter),*, Queries>
            System<'closure, ($($letter),*), Queries, &'a Resources, Resources> for Closure
        where
            Closure: FnMut(SystemContext, ($($letter),*), Queries) + 'closure,
            Closure: System<'closure, ($($letter),*), Queries, ($($letter),*), ()>,
            $(for<'r> $letter: FetchForSystem<'r>,)*
            Queries: QueryBundle,
        {
            #[allow(non_snake_case)]
            fn run(&mut self, world: &World, resources: &'a Resources) {
                let ($(mut $letter,)*) = ($($letter::fetch(resources),)*);
                self.run(world, ($($letter::deref(&mut $letter),)*));
            }
        }
    }
}

impl_for_tuples!(impl_system);

#[test]
fn smoke_test() {
    use crate::Executor;
    let mut executor = Executor::<(Mut<f32>, Ref<u32>, Ref<u64>)>::builder()
        .system(|_, _: (&mut f32, &u32), _: ()| {})
        .system(|_, _: (&mut f32, &u64), _: ()| {})
        .build();
    let world = hecs::World::new();

    let (mut a, b, c) = (1.0f32, 2u32, 3u64);
    executor.run(&world, (&mut a, &b, &c));

    let mut resources = resources::Resources::new();
    resources.insert(1.0f32);
    resources.insert(2u32);
    resources.insert(3u64);
    executor.run(&world, &resources);

    fn dummy_system(_: SystemContext, _: (), _: ()) {}
    dummy_system.run(&world, &resources);

    fn sum_system(_: SystemContext, (a, b): (&mut i32, &usize), _: ()) {
        *a += *b as i32;
    }
    resources.insert(3usize);
    resources.insert(1i32);
    sum_system.run(&world, &resources);
    assert_eq!(*resources.get::<i32>().unwrap(), 4);
    sum_system.run(&world, &resources);
    assert_eq!(*resources.get::<i32>().unwrap(), 7);
}

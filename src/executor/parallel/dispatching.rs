use parking_lot::Mutex;
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

use super::SystemClosure;
use crate::{ResourceTuple, SystemId};

/// Parallel executor variant, used when all systems are proven to be statically disjoint,
/// and have no dependencies.
pub struct Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub systems: HashMap<SystemId, Arc<Mutex<SystemClosure<'closures, Resources::Wrapped>>>>,
}

impl<'closures, Resources> Dispatcher<'closures, Resources>
where
    Resources: ResourceTuple,
{
    pub fn run(&mut self, world: &hecs::World, wrapped: Resources::Wrapped) {
        // All systems are statically disjoint, so they can all be running together at all times.
        self.systems.par_iter().for_each(|(_, system)| {
            let system = &mut *system
                .try_lock() // TODO should this be .lock() instead?
                .expect("systems should only be ran once per execution");
            system(world, &wrapped);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::ExecutorParallel;
    use crate::{
        resource::{AtomicBorrow, WrappableSingle},
        Executor, Mut, Query, Ref,
    };
    use hecs::World;

    struct A(usize);
    struct B(usize);
    struct C(usize);

    #[test]
    fn trivial() {
        ExecutorParallel::<()>::build(
            Executor::builder()
                .system(|_: (), _: ()| {})
                .system(|_: (), _: ()| {}),
        )
        .unwrap_to_dispatcher();
    }

    #[test]
    fn trivial_with_resources() {
        ExecutorParallel::<(Ref<A>, Ref<B>, Ref<C>)>::build(
            Executor::builder()
                .system(|_: (), _: ()| {})
                .system(|_: (), _: ()| {}),
        )
        .unwrap_to_dispatcher();
    }

    #[test]
    fn resources_disjoint() {
        let world = World::new();
        let mut a = A(0);
        let mut b = B(1);
        let c = C(2);
        let mut executor = ExecutorParallel::<(Mut<A>, Mut<B>, Ref<C>)>::build(
            Executor::builder()
                .system(|(a, c): (&mut A, &C), _: ()| {
                    a.0 += c.0;
                })
                .system(|(b, c): (&mut B, &C), _: ()| {
                    b.0 += c.0;
                }),
        )
        .unwrap_to_dispatcher();
        let mut borrows = (
            AtomicBorrow::new(),
            AtomicBorrow::new(),
            AtomicBorrow::new(),
        );
        let wrapped = (
            wrap_helper!(mut a, A, borrows.0),
            wrap_helper!(mut b, B, borrows.1),
            wrap_helper!(c, C, borrows.2),
        );
        executor.run(&world, wrapped);
        assert_eq!(a.0, 2);
        assert_eq!(b.0, 3);
    }

    #[test]
    fn components_disjoint() {
        let mut world = World::new();
        world.spawn_batch((0..10).map(|_| (A(0), B(0), C(0))));
        let a = A(1);
        let mut executor = ExecutorParallel::<Ref<A>>::build(
            Executor::builder()
                .system(|a: &A, q: Query<(&A, &mut B)>| {
                    for (_, (_, b)) in q.query().iter() {
                        b.0 += a.0;
                    }
                })
                .system(|a: &A, q: Query<(&A, &mut C)>| {
                    for (_, (_, c)) in q.query().iter() {
                        c.0 += a.0;
                    }
                }),
        )
        .unwrap_to_dispatcher();
        let mut borrow = AtomicBorrow::new();
        let wrapped = (wrap_helper!(a, A, borrow),);
        executor.run(&world, wrapped);
        for (_, (b, c)) in world.query::<(&B, &C)>().iter() {
            assert_eq!(b.0, 1);
            assert_eq!(c.0, 1);
        }
    }
}

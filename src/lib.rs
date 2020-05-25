//! [![Latest Version]][crates.io]
//! [![Documentation]][docs.rs]
//! [![Dependencies]][deps.rs]
//! [![License]][license link]
//!
//! `yaks` aims to be a minimalistic, yet featureful and performant systems framework for [`hecs`],
//! with optional parallel execution. It is built upon [`hecs`] and [`resources`].
//!
//! The goals are, in no particular order:
//! - safety
//! - simplicity
//! - performance
//! - extensibility
//! - tight engineering
//! - minimal dependencies
//! - effortless concurrency
//!
//! This is a very early version. It's API is subject to radical change, it does not do any
//! multithreading, or system ordering beyond insertion order.
//!
//! # Example
//! ```rust
//! use hecs::World;
//! use resources::Resources;
//! use yaks::{Executor, ModQueuePool, System};
//!
//! struct Position(f32);
//! struct Velocity(f32);
//! struct Acceleration(f32);
//! struct HighestVelocity(f32);
//!
//! let mut world = World::new();
//! let mut resources = Resources::new();
//! let mod_queues = ModQueuePool::new();
//! world.spawn((Position(0.0), Velocity(3.0)));
//! world.spawn((Position(0.0), Velocity(1.0), Acceleration(1.0)));
//! resources.insert(HighestVelocity(0.0));
//!
//! let motion = System::builder()
//!     .query::<(&mut Position, &Velocity)>()
//!     .query::<(&mut Velocity, &Acceleration)>()
//!     .build(|facade, _, (q_1, q_2)| {
//!         for (_, (mut pos, vel)) in facade.query(q_1).iter() {
//!             pos.0 += vel.0;
//!         }
//!         for (_, (mut vel, acc)) in facade.query(q_2).iter() {
//!             vel.0 += acc.0;
//!         }
//!     });
//!
//! let find_highest = System::builder()
//!     .resources::<&mut HighestVelocity>()
//!     .query::<&Velocity>()
//!     .build(|facade, mut highest, query| {
//!         for (_, vel) in facade.query(query).iter() {
//!             if vel.0 > highest.0 {
//!                 highest.0 = vel.0;
//!             }
//!         }
//!     });
//!
//! let mut executor = Executor::<()>::builder()
//!     .system(motion)
//!     .system(find_highest)
//!     .build();
//! assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 0.0);
//! executor.run(&world, &resources, &mod_queues);
//! assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
//! executor.run(&world, &resources, &mod_queues);
//! assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 3.0);
//! executor.run(&world, &resources, &mod_queues);
//! assert_eq!(resources.get::<HighestVelocity>().unwrap().0, 4.0);
//! ```
//!
//! [`hecs`]: https://crates.io/crates/hecs
//! [`resources`]: https://crates.io/crates/resources
//!
//! [Latest Version]: https://img.shields.io/crates/v/yaks.svg
//! [crates.io]: https://crates.io/crates/yaks
//! [Documentation]: https://docs.rs/yaks/badge.svg
//! [docs.rs]: https://docs.rs/yaks
//! [Dependencies]: https://deps.rs/repo/github/Ratysz/yaks/status.svg
//! [deps.rs]: https://deps.rs/repo/github/Ratysz/yaks
//! [License]: https://img.shields.io/crates/l/yaks.svg
//! [license link]: https://github.com/Ratysz/yaks/blob/master/LICENSE.md

// TODO uncomment #![warn(missing_docs)]

#[macro_use]
mod tuple_macro;

#[cfg(feature = "parallel")]
mod access_set;
mod atomic_borrow;
mod batch_helper;
mod contains;
mod deref_tuple;
mod executor;
mod executor_builder;
mod fetch;
mod query_bundle;
mod resource_cell;
mod resource_tuple;
mod system_context;

#[cfg(feature = "parallel")]
use access_set::{ArchetypeSet, ComponentSet, ComponentTypeSet, ResourceSet, TypeSet};
use atomic_borrow::AtomicBorrow;
use contains::Contains;
use deref_tuple::DerefTuple;
use executor::SystemClosure;
use executor_builder::SystemId;
use fetch::Fetch;
use query_bundle::QueryBundle;
use resource_cell::{Ref, RefMut, ResourceCell};
use resource_tuple::{ResourceTuple, ResourceWrap, WrappedResources};

pub use batch_helper::batch;
pub use executor::Executor;
pub use executor_builder::ExecutorBuilder;
pub use query_bundle::QueryMarker;
pub use system_context::SystemContext;

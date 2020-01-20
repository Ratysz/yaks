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
//! let mut executor = Executor::<()>::new().with(motion).with(find_highest);
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

pub use hecs;
pub use resources;
#[cfg(feature = "impl_scoped_threadpool")]
pub use scoped_threadpool;

pub use hecs::{Entity, World};
pub use resources::Resources;

mod borrows;
mod error;
mod executor;
#[cfg(feature = "parallel")]
mod executor_parallel_impls;
mod impls_for_tuple;
mod mod_queue;
mod query_bundle;
mod resource_bundle;
mod system;
mod system_container;
mod world_facade;

pub use error::NoSuchSystem;
pub use executor::Executor;
#[cfg(feature = "parallel")]
pub use executor_parallel_impls::ThreadpoolScope;
pub use mod_queue::{ModQueue, ModQueuePool};
pub use system::{System, SystemBuilder};
pub use world_facade::WorldFacade;

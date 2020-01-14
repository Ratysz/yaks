//! [![Latest Version]][crates.io]
//! [![Documentation]][docs.rs]
//! [![Dependencies]][deps.rs]
//! [![License]][license link]
//!
//! `yaks` aims to be a minimalistic, yet featureful and performant
//! entity-component-system (ECS) framework. It is built upon [`hecs`] and [`resources`].
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
//! use yaks::{System, Executor, World, Entity};
//!
//! struct Position(f32);
//! struct Velocity(f32);
//! struct Acceleration(f32);
//! struct HighestVelocity(f32);
//!
//! let mut world = World::new();
//! world.add_resource(HighestVelocity(0.0));
//! world.spawn((Position(0.0), Velocity(3.0)));
//! world.spawn((Position(0.0), Velocity(1.0), Acceleration(1.0)));
//!
//! let motion = System::builder()
//!     .query::<(&mut Position, &Velocity)>()
//!     .query::<(&mut Velocity, &Acceleration)>()
//!     .build(|world, _, (q_1, q_2)| {
//!         for (_, (mut pos, vel)) in q_1.query(world).iter() {
//!             pos.0 += vel.0;
//!         }
//!         for (_, (mut vel, acc)) in q_2.query(world).iter() {
//!             vel.0 += acc.0;
//!         }
//!     });
//!
//! let find_highest = System::builder()
//!     .resources::<&mut HighestVelocity>()
//!     .query::<&Velocity>()
//!     .build(|world, mut highest, query| {
//!         for (_, vel) in query.query(world).iter() {
//!             if vel.0 > highest.0 {
//!                 highest.0 = vel.0;
//!             }
//!         }
//!     });
//!
//! let mut executor = Executor::<()>::new().with(motion).with(find_highest);
//! assert_eq!(world.fetch::<&HighestVelocity>().0, 0.0);
//! executor.run(&mut world);
//! assert_eq!(world.fetch::<&HighestVelocity>().0, 3.0);
//! executor.run(&mut world);
//! assert_eq!(world.fetch::<&HighestVelocity>().0, 3.0);
//! executor.run(&mut world);
//! assert_eq!(world.fetch::<&HighestVelocity>().0, 4.0);
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

#![warn(missing_docs)]

#[doc(hidden)]
pub use hecs::{
    Bundle as ComponentBundle, DynamicBundle as DynamicComponentBundle, EntityRef as Components,
    Query, Ref as ComponentRef, RefMut as ComponentRefMut,
};
#[doc(hidden)]
pub use resources::{Entry as ResourceEntry, Ref as ResourceRef, RefMut as ResourceRefMut};

pub use hecs::{Component, Entity, QueryBorrow};
pub use resources::Resource;

pub mod error;
mod executor;
mod impls_for_tuple;
mod mod_queue;
mod query_bundle;
mod resource_bundle;
mod system;
mod world;

pub use executor::Executor;
pub use mod_queue::ModQueue;
pub use system::{System, SystemBuilder};
pub use world::World;

//! Resource flow:
//! - `Source` argument is passed to `Executor::<Resources: ResourceTuple + WrappableTuple>::run()`,
//! - tuple of intermediates (`WrappableTuple::Intermediates`) is extracted from the argument
//! (`WrappableTuple::get()`; `MarkerGet` for remote-implementing that on `Copy` sources),
//! - the references, together with `AtomicBorrow`s from the executor,
//! are wrapped into `ResourceCell`s (`WrappableTuple::wrap()`),
//! - when each system in the executor is ran, a subset tuple of references matching
//! that of the system's resources argument is fetched from the cells, setting runtime
//! borrow checking (`Fetch` for the whole tuple, `Contains` for each of its elements),
//! - the subset tuple of references is passed into the system's boxed closure,
//! - after closure returns, the borrows are "released", resetting runtime
//! borrow checking (`Fetch` and `Contains` again),
//! - after all of the systems have been ran, the cells are dropped.

mod atomic_borrow;
mod cell;
mod contains;
#[cfg(feature = "resources-interop")]
mod resources_interop;
mod tuple;
mod wrap;

use cell::{ResourceMutCell, ResourceRefCell};

pub use atomic_borrow::AtomicBorrow;
pub use contains::{ContainsMut, ContainsRef};
pub use tuple::{Mut, Ref, ResourceTuple};
pub use wrap::{MarkerGet, WrappableSingle, WrappableTuple};

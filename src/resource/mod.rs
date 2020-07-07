//! Resource flow:
//! - resources argument is passed to `Executor::<Tuple: ResourceTuple>::run()`,
//! - tuple of references to types in `Tuple` is extracted
//! from the argument (`RefExtractor`),
//! - the references, together with `AtomicBorrow`s from the executor,
//! are wrapped into `ResourceCell`s (`ResourceWrap`),
//! - when each system in the executor is ran, a subset tuple of references matching
//! that of the system's resources argument is fetched from the cells, setting runtime
//! borrow checking (`Fetch` for the whole tuple, `Contains` for each of it's elements),
//! - the subset tuple of references is passed into the system's boxed closure,
//! - after closure returns, the borrows are "released", resetting runtime
//! borrow checking (`Fetch` and `Contains` again),
//! - after all of the systems have been ran, the cells are dropped.

mod atomic_borrow;
mod cell;
mod contains;
mod fetch;
mod ref_extractor;
mod tuple;
mod wrap;

use cell::ResourceCell;
use contains::Contains;

pub use atomic_borrow::AtomicBorrow;
pub use fetch::Fetch;
pub use ref_extractor::RefExtractor;
pub use tuple::ResourceTuple;
pub use wrap::ResourceWrap;

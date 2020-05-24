#[macro_use]
mod tuple_macro;

mod atomic_borrow;
#[cfg(feature = "parallel")]
mod borrow_set;
mod contains;
mod deref_tuple;
mod executor;
mod executor_builder;
mod fetch;
mod query_bundle;
mod resource_cell;
mod resource_tuple;
mod system_context;

use atomic_borrow::AtomicBorrow;
#[cfg(feature = "parallel")]
use borrow_set::{ArchetypeSet, ComponentSet, ComponentTypeSet, ResourceSet, TypeSet};
use contains::Contains;
use deref_tuple::DerefTuple;
use executor::SystemClosure;
use executor_builder::SystemId;
use fetch::Fetch;
use query_bundle::QueryBundle;
use resource_cell::{Ref, RefMut, ResourceCell};
use resource_tuple::{ResourceTuple, ResourceWrap, WrappedResources};

pub use executor::Executor;
pub use executor_builder::ExecutorBuilder;
pub use query_bundle::QueryMarker;
pub use system_context::SystemContext;

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

/// Error indicating that no [`System`] with the provided handle is present in an [`Executor`].
///
/// [`System`]: struct.System.html
/// [`Executor`]: struct.Executor.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NoSuchSystem;

impl Display for NoSuchSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "no such system")
    }
}

impl Error for NoSuchSystem {}

#[derive(Debug, Eq, PartialEq)]
pub enum CantInsertSystem {
    CyclicDependency,
    DependencyNotFound(String),
}

impl Display for CantInsertSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CantInsertSystem::CyclicDependency => write!(
                f,
                "inserting the system would create a cycle of dependencies"
            ),
            CantInsertSystem::DependencyNotFound(handle) => write!(
                f,
                "invalid dependency: no system with handle `{:?}`",
                handle
            ),
        }
    }
}

impl Error for CantInsertSystem {}

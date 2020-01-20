use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// Error indicating that no [`System`] with the provided handle is present in an [`Executor`].
///
/// [`System`]: struct.System.html
/// [`Executor`]: struct.Executor.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchSystem;

impl Display for NoSuchSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad("no such system")
    }
}

impl Error for NoSuchSystem {}

/*#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CyclicDependency;

impl Display for CyclicDependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad("adding the system would create an unresolvable cycle")
    }
}

impl Error for CyclicDependency {}
*/

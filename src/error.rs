use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NonUniqueSystemHandle;

impl Display for NonUniqueSystemHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad("system handles can only appear once in an executor")
    }
}

impl Error for NonUniqueSystemHandle {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchSystem;

impl Display for NoSuchSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad("no such system")
    }
}

impl Error for NoSuchSystem {}

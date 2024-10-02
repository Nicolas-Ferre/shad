use std::fmt::{Display, Formatter};

/// An analysis result.
pub type Result<T> = std::result::Result<T, Error>;

/// An analysis error.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed code analysis")
    }
}

impl std::error::Error for Error {}

pub(crate) fn result_ref<T>(result: &Result<T>) -> Result<&T> {
    result.as_ref().map_err(|&err| err)
}

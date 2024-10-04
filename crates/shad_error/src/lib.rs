//! Errors for the Shad programming language.
//!
//! This crate provides error types used by Shad parser and analyzer.

use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::{error, io};

mod semantic;
mod span;
mod syntax;

pub use semantic::*;
pub use span::*;
pub use syntax::*;

/// An error returned when during Shad code compilation before running it.
#[derive(Debug)]
pub enum Error {
    /// A syntax error
    Syntax(SyntaxError),
    /// Semantic errors.
    Semantic(Vec<SemanticError>),
    /// An I/O error.
    Io(io::Error),
}

// coverage: off (not critical logic)
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(err) => writeln!(f, "{err}"),
            Self::Semantic(err) => {
                let sorted_errors = err
                    .iter()
                    .sorted_unstable_by_key(|err| err.located_messages[0].span.start)
                    .map(|err| format!("{err}"))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                writeln!(f, "{sorted_errors}")
            }
            Self::Io(err) => writeln!(f, "{err}"),
        }
    }
}
// coverage: on

impl error::Error for Error {}

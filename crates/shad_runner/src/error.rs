use shad_analyzer::SemanticError;
use shad_parser::SyntaxError;
use std::fmt::{Display, Formatter};
use std::{error, io};

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
            Self::Syntax(err) => Display::fmt(err, f),
            Self::Semantic(err) => {
                for err in err {
                    writeln!(f, "{err}")?;
                }
                Ok(())
            }
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}
// coverage: on

impl error::Error for Error {}

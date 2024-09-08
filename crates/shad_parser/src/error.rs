use annotate_snippets::{Level, Renderer, Snippet};
use std::fmt::{Display, Formatter};
use std::{error, io};

/// An error obtained when trying to parse a Shad code.
#[derive(Debug)]
pub enum Error {
    /// A parsing error.
    Syntax(SyntaxError),
    /// An I/O error.
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Syntax(err1), Self::Syntax(err2)) => err1 == err2,
            (Self::Io(err1), Self::Io(err2)) => err1.to_string() == err2.to_string(),
            _ => false,
        }
    }
}

impl Eq for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(err) => Display::fmt(err, f),
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl error::Error for Error {}

/// A syntax error obtained when trying to parse a Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// The byte offset where the error is located in the file.
    pub offset: usize,
    /// The error message.
    pub message: String,
    /// The formatted error message.
    pub pretty_message: String,
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}

impl error::Error for SyntaxError {}

impl SyntaxError {
    pub(crate) fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
            pretty_message: String::new(),
        }
    }

    #[allow(clippy::range_plus_one)]
    pub(crate) fn with_pretty_message(self, file_path: &str, code: &str) -> Self {
        let message = Level::Error.title(&self.message).snippet(
            Snippet::source(code)
                .fold(true)
                .origin(file_path)
                .annotation(
                    Level::Error
                        .span(self.offset.min(code.len() - 1)..(self.offset + 1).min(code.len()))
                        .label("here"),
                ),
        );
        let pretty_message = format!("{}", Renderer::styled().render(message));
        Self {
            offset: self.offset,
            message: self.message,
            pretty_message,
        }
    }
}

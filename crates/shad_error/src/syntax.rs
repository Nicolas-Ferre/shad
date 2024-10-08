use annotate_snippets::{Level, Renderer, Snippet};
use std::error;
use std::fmt::{Display, Formatter};

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

// coverage: off (not critical logic)
impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}
// coverage: on

impl error::Error for SyntaxError {}

impl SyntaxError {
    /// Creates a syntax error.
    pub fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
            pretty_message: String::new(),
        }
    }

    /// Generates the formatted error string.
    #[allow(clippy::range_plus_one)]
    pub fn with_pretty_message(self, file_path: &str, code: &str) -> Self {
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

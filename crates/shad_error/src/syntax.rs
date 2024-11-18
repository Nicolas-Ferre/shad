use crate::{ModuleLocation, Span};
use annotate_snippets::{Level, Renderer, Snippet};
use std::error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// A syntax error obtained when trying to parse a Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// The byte offset where the error is located in the file.
    pub span: Span,
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
    pub fn new(offset: usize, module: Rc<ModuleLocation>, message: impl Into<String>) -> Self {
        let message = message.into();
        let start = offset.min(module.code.len().saturating_sub(1));
        let end = offset
            + module.code[offset..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
        Self {
            pretty_message: format!(
                "{}",
                Renderer::styled().render(
                    Level::Error.title(&message).snippet(
                        Snippet::source(&module.code)
                            .fold(true)
                            .origin(&module.path)
                            .annotation(Level::Error.span(start..end).label("here")),
                    )
                )
            ),
            span: Span::new(start, end, module),
            message,
        }
    }
}

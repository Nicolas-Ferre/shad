use annotate_snippets::{Level, Renderer, Snippet};
use shad_parser::Span;
use std::error;
use std::fmt::{Display, Formatter};

/// A semantic error obtained when analyzing a parsed Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    /// Main error message.
    pub message: String,
    /// Located messages to improve debugging.
    pub located_messages: Vec<LocatedMessage>,
    /// The formatted error message.
    pub pretty_message: String,
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}

impl error::Error for SemanticError {}

impl SemanticError {
    pub(crate) fn new(
        message: impl Into<String>,
        located_messages: Vec<LocatedMessage>,
        parsed: &shad_parser::ParsedProgram,
    ) -> Self {
        let mut snippet = Snippet::source(&parsed.code)
            .fold(true)
            .origin(&parsed.path);
        for message in &located_messages {
            let start = message.span.start.min(parsed.code.len());
            let end = message.span.end.min(parsed.code.len());
            snippet = snippet.annotation(
                Level::from(message.level)
                    .span(start..end)
                    .label(&message.text),
            );
        }
        let message = message.into();
        let pretty_message = format!(
            "{}",
            Renderer::styled().render(Level::Error.title(&message).snippet(snippet))
        );
        Self {
            message,
            located_messages,
            pretty_message,
        }
    }
}

/// A located message to help debugging a semantic error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedMessage {
    /// The message level.
    pub level: ErrorLevel,
    /// The message span.
    pub span: Span,
    /// The message text.
    pub text: String,
}

/// The level of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    /// An error.
    Error,
    /// An information.
    Info,
}

impl From<ErrorLevel> for Level {
    fn from(value: ErrorLevel) -> Self {
        match value {
            ErrorLevel::Error => Self::Error,
            ErrorLevel::Info => Self::Info,
        }
    }
}

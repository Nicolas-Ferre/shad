use crate::Span;
use annotate_snippets::{Level, Renderer, Snippet};
use itertools::Itertools;
use std::error;
use std::fmt::{Display, Formatter};

/// A semantic error obtained when analyzing a Shad AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    /// Main error message.
    pub message: String,
    /// Located messages to improve debugging.
    pub located_messages: Vec<LocatedMessage>,
    /// The formatted error message.
    pub pretty_message: String,
}

// coverage: off (not critical logic)
impl Display for SemanticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}
// coverage: on

impl error::Error for SemanticError {}

impl SemanticError {
    /// Creates a semantic error.
    pub fn new(message: impl Into<String>, located_messages: Vec<LocatedMessage>) -> Self {
        let message = message.into();
        let snippets = located_messages
            .iter()
            .into_group_map_by(|message| &message.span.module)
            .into_iter()
            .sorted_unstable_by_key(|(_, messages)| messages[0].level)
            .map(|(module, messages)| {
                let mut snippet = Snippet::source(&module.code)
                    .fold(true)
                    .origin(&module.path);
                for message in &messages {
                    let start = message.span.start.min(module.code.len());
                    let end = message.span.end.min(module.code.len());
                    snippet = snippet.annotation(
                        Level::from(message.level)
                            .span(start..end)
                            .label(&message.text),
                    );
                }
                snippet
            });
        Self {
            pretty_message: format!(
                "{}",
                Renderer::styled().render(Level::Error.title(&message).snippets(snippets))
            ),
            message,
            located_messages,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

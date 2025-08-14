use annotate_snippets::{Level, Renderer, Snippet};
use itertools::Itertools;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A Shad compilation error.
#[derive(Debug)]
pub enum Error {
    /// An I/O error.
    Io(Vec<(PathBuf, io::Error)>),
    /// A parsing error.
    Parsing(Vec<ParsingError>),
    /// A validation error.
    Validation(Vec<ValidationError>),
}

impl Error {
    /// Renders the error.
    #[allow(suspicious_double_ref_op)]
    pub fn render(&self) -> String {
        match self {
            Self::Io(errors) => errors
                .iter()
                .map(|(path, error)| (path, Self::render_io(path, error)))
                .sorted_unstable_by_key(|(path, message)| (path.clone(), message.clone()))
                .map(|(_, message)| message)
                .join("\n\n"),
            Self::Parsing(errors) => errors
                .iter()
                .map(|error| (error, Self::render_parsing(error)))
                .sorted_unstable_by_key(|(error, message)| {
                    (error.path.clone(), error.offset, message.clone())
                })
                .map(|(_, message)| message)
                .join("\n\n"),
            Self::Validation(errors) => errors
                .iter()
                .map(|error| (error, Self::render_validation(error)))
                .sorted_unstable_by_key(|(error, message)| {
                    (error.path.clone(), error.span.start, message.clone())
                })
                .map(|(_, message)| message)
                .join("\n\n"),
        }
    }

    fn render_io(path: &Path, error: &io::Error) -> String {
        Renderer::styled()
            .render(Level::Error.title(&format!("{}: {error}", path.display())))
            .to_string()
    }

    fn render_parsing(error: &ParsingError) -> String {
        let expected_tokens = error
            .expected_tokens
            .iter()
            .enumerate()
            .map(|(index, token)| {
                if index == 0 {
                    token.to_string()
                } else if index == error.expected_tokens.len() - 1 {
                    format!(" or {token}")
                } else {
                    format!(", {token}")
                }
            })
            .join("");
        let message = format!("expected {expected_tokens}");
        let path = error.path.display().to_string();
        let renderer = Renderer::styled();
        let content = renderer.render(
            Level::Error.title(&message).snippet(
                Snippet::source(&error.code)
                    .fold(true)
                    .origin(&path)
                    .annotation(Level::Error.span(error.offset..error.offset).label("here")),
            ),
        );
        content.to_string()
    }

    fn render_validation(error: &ValidationError) -> String {
        let path = error.path.display().to_string();
        let renderer = Renderer::styled();
        let level = Self::convert_level(&error.level);
        let mut annotations = vec![level.span(error.span.clone())];
        for inner in &error.inner {
            if inner.path == error.path {
                annotations.push(
                    Self::convert_level(&inner.level)
                        .span(inner.span.clone())
                        .label(&inner.message),
                );
            }
        }
        let mut snippets = vec![Snippet::source(&error.code)
            .fold(true)
            .origin(&path)
            .annotations(annotations)];
        // coverage: off (unused for now)
        for inner in &error.inner {
            if inner.path != error.path {
                snippets.push(
                    Snippet::source(&error.code)
                        .fold(true)
                        .origin(&path)
                        .annotation(
                            Self::convert_level(&inner.level)
                                .span(inner.span.clone())
                                .label(&inner.message),
                        ),
                );
            }
        }
        // coverage: on
        let content = renderer.render(level.title(&error.message).snippets(snippets));
        content.to_string()
    }

    fn convert_level(level: &ValidationMessageLevel) -> Level {
        match level {
            ValidationMessageLevel::Error => Level::Error,
            ValidationMessageLevel::Info => Level::Info,
        }
    }
}

/// A parsing error.
#[derive(Debug)]
pub struct ParsingError {
    /// The tokens that are expected at the location of the error.
    pub expected_tokens: Vec<String>,
    /// The offset of the error in the file.
    pub offset: usize,
    /// The file content.
    pub code: String,
    /// The file path.
    pub path: PathBuf,
    pub(crate) forced: bool,
}

/// A validation error.
#[derive(Debug)]
pub struct ValidationError {
    /// The validation message level.
    pub level: ValidationMessageLevel,
    /// The validation message.
    pub message: String,
    /// The span where the error is located.
    pub span: Range<usize>,
    /// The file content.
    pub code: String,
    /// The file path.
    pub path: PathBuf,
    /// Inner errors providing more details.
    pub inner: Vec<ValidationError>,
}

/// A validation message level.
#[derive(Debug)]
pub enum ValidationMessageLevel {
    /// An error.
    Error,
    /// An information.
    Info,
}

use annotate_snippets::{Level, Renderer, Snippet};
use itertools::Itertools;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Io(Vec<(PathBuf, io::Error)>),
    Parsing(Vec<ParsingError>),
    Validation(Vec<ValidationError>),
}

impl Error {
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
        let content = renderer.render(level.title(&error.message).snippets(snippets));
        content.to_string()
    }

    fn convert_level(level: &ValidationErrorLevel) -> Level {
        match level {
            ValidationErrorLevel::Error => Level::Error,
            ValidationErrorLevel::Info => Level::Info,
        }
    }
}

#[derive(Debug)]
pub struct ParsingError {
    pub expected_tokens: Vec<String>,
    pub offset: usize,
    pub code: String,
    pub path: PathBuf,
    pub(crate) forced: bool,
}

#[derive(Debug)]
pub struct ValidationError {
    pub level: ValidationErrorLevel,
    pub message: String,
    pub span: Range<usize>,
    pub code: String,
    pub path: PathBuf,
    pub inner: Vec<ValidationError>,
}

#[derive(Debug)]
pub enum ValidationErrorLevel {
    Error,
    Info,
}

use crate::compilation::node::Node;
use crate::compilation::validation::ValidationContext;
use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
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
        Renderer::styled().render(&[Group::with_title(
            Level::ERROR.primary_title(format!("{}: {error}", path.display())),
        )])
    }

    fn render_parsing(error: &ParsingError) -> String {
        let expected_tokens = error
            .expected_tokens
            .iter()
            .sorted_unstable()
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
        renderer.render(&[
            Group::with_title(Level::ERROR.primary_title(&message)).element(
                Snippet::source(&error.code)
                    .fold(true)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(error.offset..error.offset)
                            .label("here"),
                    ),
            ),
        ])
    }

    fn render_validation(error: &ValidationError) -> String {
        let path = error.path.display().to_string();
        let renderer = Renderer::styled();
        let mut annotations =
            vec![Self::annotation_level(error.level, error.level).span(error.span.clone())];
        for inner in &error.inner {
            if inner.path == error.path {
                annotations.push(
                    Self::annotation_level(error.level, inner.level)
                        .span(inner.span.clone())
                        .label(&inner.message),
                );
            }
        }
        let mut snippets = vec![Snippet::source(&error.code)
            .fold(true)
            .path(&path)
            .annotations(annotations)];
        // coverage: off (unused for now)
        for inner in &error.inner {
            if inner.path != error.path {
                snippets.push(
                    Snippet::source(&error.code)
                        .fold(true)
                        .path(&path)
                        .annotation(
                            Self::annotation_level(error.level, inner.level)
                                .span(inner.span.clone())
                                .label(&inner.message),
                        ),
                );
            }
        }
        // coverage: on
        renderer.render(&[
            Group::with_title(Level::ERROR.primary_title(&error.message)).elements(snippets),
        ])
    }

    fn annotation_level(
        base_level: ValidationMessageLevel,
        level: ValidationMessageLevel,
    ) -> AnnotationKind {
        if level <= base_level {
            AnnotationKind::Primary
        } else {
            AnnotationKind::Context
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

impl ValidationError {
    pub(crate) fn error(
        ctx: &ValidationContext<'_>,
        node: &dyn Node,
        title: &str,
        label: Option<&str>,
        secondary: &[(&dyn Node, &str)],
    ) -> Self {
        Self {
            level: ValidationMessageLevel::Primary,
            message: title.into(),
            span: node.span.clone(),
            code: ctx.roots[&node.path].slice.clone(),
            path: node.path.clone(),
            inner: label
                .map(|label| Self::simple(ctx, ValidationMessageLevel::Primary, node, label))
                .into_iter()
                .chain(secondary.iter().map(|&(node, label)| {
                    Self::simple(ctx, ValidationMessageLevel::Context, node, label)
                }))
                .collect(),
        }
    }

    fn simple(
        ctx: &ValidationContext<'_>,
        level: ValidationMessageLevel,
        node: &dyn Node,
        label: &str,
    ) -> Self {
        Self {
            level,
            message: label.into(),
            span: node.span.clone(),
            code: ctx.roots[&node.path].slice.clone(),
            path: node.path.clone(),
            inner: vec![],
        }
    }
}

/// A validation message level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationMessageLevel {
    /// An error.
    Primary,
    /// An information.
    Context,
}

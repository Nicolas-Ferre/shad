#![allow(missing_docs)] // TODO: remove

mod ast;
mod config;
mod functions;
mod index;
mod parsing;
mod reading;
mod transpilation;
mod validation;

use crate::index::AstNodeIndex;
use crate::reading::SourceFolder;
use annotate_snippets::{Level, Renderer, Snippet};
pub use ast::*;
use itertools::Itertools;
pub use parsing::*;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
pub use validation::*;

const FILE_EXT: &str = "shd";

pub fn compile(folder: impl SourceFolder) -> Result<Vec<String>, Error> {
    let config = config::load_config().expect("internal error: config should be valid");
    let root_path = folder.path();
    let files = reading::read_files(folder).map_err(Error::Io)?;
    let mut asts = parse_files(&config, &files)
        .map_err(Error::Parsing)?
        .into_iter()
        .map(|(path, (code, ast))| {
            (
                path,
                FileAst {
                    code,
                    index: AstNodeIndex::new(&ast),
                    root: ast,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    AstNodeIndex::generate_lookup_paths(&config, &mut asts, &root_path);
    validate_asts(&asts, &root_path).map_err(Error::Validation)?;
    Ok(transpilation::transpile_asts(&config, &asts))
}

#[derive(Debug)]
pub struct FileAst {
    code: String,
    index: AstNodeIndex,
    root: Rc<AstNode>,
}

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
                .collect(),
            Self::Parsing(errors) => errors
                .iter()
                .map(|error| (error, Self::render_parsing(error)))
                .sorted_unstable_by_key(|(error, message)| {
                    (error.path.clone(), error.offset, message.clone())
                })
                .map(|(_, message)| message)
                .collect(),
            Self::Validation(errors) => errors
                .iter()
                .map(|error| (error, Self::render_validation(error)))
                .sorted_unstable_by_key(|(error, message)| {
                    (error.path.clone(), error.span.start, message.clone())
                })
                .map(|(_, message)| message)
                .collect(),
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

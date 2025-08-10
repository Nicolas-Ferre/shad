use crate::ast::{AstNode, AstNodeInner};
use crate::functions::validation;
use crate::FileAst;
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

pub(crate) fn validate_asts(
    asts: &HashMap<PathBuf, FileAst>,
    root_path: &Path,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    for path in asts.keys() {
        if let Err(err) = validate_ast(asts, root_path, path) {
            errors.extend(err);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_ast(
    asts: &HashMap<PathBuf, FileAst>,
    root_path: &Path,
    path: &Path,
) -> Result<(), Vec<ValidationError>> {
    let mut ctx = ValidationContext {
        path,
        root_path,
        asts,
        errors: vec![],
    };
    validate_ast_node(&mut ctx, &asts[path].root);
    if ctx.errors.is_empty() {
        Ok(())
    } else {
        Err(ctx.errors)
    }
}

fn validate_ast_node(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    match &node.inner {
        AstNodeInner::Sequence(children) => {
            for child in children.values() {
                validate_ast_node(ctx, child);
            }
        }
        AstNodeInner::Repeated(children) => {
            for child in children {
                validate_ast_node(ctx, child);
            }
        }
        AstNodeInner::Terminal => {}
    }
    for validation in &node.kind_config.validation {
        validation::run(ctx, validation, node);
    }
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

#[derive(Debug)]
pub(crate) struct ValidationContext<'a> {
    pub(crate) path: &'a Path,
    pub(crate) root_path: &'a Path,
    pub(crate) asts: &'a HashMap<PathBuf, FileAst>,
    pub(crate) errors: Vec<ValidationError>,
}

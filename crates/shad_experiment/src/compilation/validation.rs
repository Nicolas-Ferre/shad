use crate::compilation::ast::{AstNode, AstNodeInner};
use crate::compilation::error::ValidationError;
use crate::config::validation;
use crate::{Error, FileAst};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn validate_asts(
    asts: &HashMap<PathBuf, FileAst>,
    root_path: &Path,
) -> Result<(), Error> {
    let mut errors = vec![];
    for path in asts.keys() {
        if let Err(err) = validate_ast(asts, root_path, path) {
            errors.extend(err);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(errors))
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
    match &node.children {
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
pub(crate) struct ValidationContext<'a> {
    pub(crate) path: &'a Path,
    pub(crate) root_path: &'a Path,
    pub(crate) asts: &'a HashMap<PathBuf, FileAst>,
    pub(crate) errors: Vec<ValidationError>,
}

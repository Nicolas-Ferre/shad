use crate::compilation::ast::{AstNode, AstNodeInner};
use crate::compilation::error::ValidationError;
use crate::compilation::FileAst;
use crate::config::{scripts, Config};
use crate::{Error, ValidationMessageLevel};
use rhai::{Engine, AST};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn validate_asts(
    config: &Config,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    root_path: &Path,
) -> Result<(), Error> {
    let mut ctx = ValidationContext {
        root_path,
        asts,
        scripts: config
            .kinds
            .values()
            .flat_map(|kind| &kind.validation)
            .map(|validation| {
                (
                    validation.assertion.clone(),
                    scripts::compile(&validation.assertion, asts, root_path),
                )
            })
            .collect(),
        errors: vec![],
    };
    for path in asts.keys() {
        validate_ast_node(&mut ctx, &asts[path].root);
    }
    if ctx.errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(ctx.errors))
    }
}

fn validate_ast_node(ctx: &mut ValidationContext<'_>, node: &Rc<AstNode>) {
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
        let (script_ast, engine) = &ctx.scripts[&validation.assertion];
        let is_valid = scripts::run::<bool>(node, ctx.asts, script_ast, engine).unwrap_or(true);
        if !is_valid {
            ctx.errors.push(ValidationError::from_config(
                ctx,
                node,
                ValidationMessageLevel::Error,
                &validation.error,
            ));
        }
    }
}

#[derive(Debug)]
pub(crate) struct ValidationContext<'a> {
    pub(crate) root_path: &'a Path,
    pub(crate) asts: &'a Rc<HashMap<PathBuf, FileAst>>,
    pub(crate) scripts: HashMap<String, (AST, Engine)>,
    pub(crate) errors: Vec<ValidationError>,
}

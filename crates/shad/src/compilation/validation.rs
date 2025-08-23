use crate::compilation::ast::AstNode;
use crate::compilation::error::ValidationError;
use crate::compilation::FileAst;
use crate::config::scripts::ScriptContext;
use crate::config::{scripts, Config};
use crate::{Error, ValidationMessageLevel};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn validate_asts(
    config: &Rc<Config>,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    root_path: &Path,
) -> Result<(), Error> {
    let ctx = ScriptContext::new(config, asts, root_path);
    let mut errors = vec![];
    for path in asts.keys() {
        validate_ast_node(&mut errors, &ctx, &asts[path].root);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(errors))
    }
}

fn validate_ast_node(errors: &mut Vec<ValidationError>, ctx: &ScriptContext, node: &Rc<AstNode>) {
    for child in &node.children {
        validate_ast_node(errors, ctx, child);
    }
    for validation in &node.kind_config.validation {
        let is_valid =
            scripts::compile_and_run::<bool>(&validation.assertion, node, ctx).unwrap_or(true);
        if !is_valid {
            errors.push(ValidationError::from_config(
                ctx,
                node,
                ValidationMessageLevel::Error,
                &validation.error,
            ));
        }
    }
}

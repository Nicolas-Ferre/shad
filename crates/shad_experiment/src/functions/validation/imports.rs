use crate::validation::ValidationContext;
use crate::{AstNode, ValidationError, ValidationErrorLevel};

pub(crate) fn check_import_path(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    let path = node.import_path(ctx.path, ctx.root_path);
    if !ctx.asts.contains_key(&path) {
        ctx.errors.push(ValidationError {
            level: ValidationErrorLevel::Error,
            message: "imported file not found".into(),
            span: node.span(),
            code: ctx.asts[ctx.path].code.clone(),
            path: ctx.path.into(),
            inner: vec![ValidationError {
                level: ValidationErrorLevel::Error,
                message: format!("no file found at `{}`", path.display()),
                span: node.span(),
                code: ctx.asts[ctx.path].code.clone(),
                path: ctx.path.into(),
                inner: vec![],
            }],
        });
    }
}

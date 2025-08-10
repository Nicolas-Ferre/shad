use crate::config::ValidationConfig;
use crate::validation::ValidationContext;
use crate::{AstNode, ValidationError, ValidationErrorLevel};

pub(crate) fn check_expr_type(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let expr = node.child(&validation.params["expr"]);
    let expected = node.child(&validation.params["expected"]);
    let Some(expr_type) = expr.type_(ctx.asts, ctx.path) else {
        return;
    };
    let Some(expected_type) = expected.type_(ctx.asts, ctx.path) else {
        return;
    };
    if expr_type != expected_type {
        ctx.errors.push(ValidationError {
            level: ValidationErrorLevel::Error,
            message: "invalid expression type".into(),
            span: expr.span(),
            code: ctx.asts[ctx.path].code.clone(),
            path: ctx.path.into(),
            inner: vec![
                ValidationError {
                    level: ValidationErrorLevel::Info,
                    message: format!("expected type is `{expected_type}`"),
                    span: expected.span(),
                    code: ctx.asts[ctx.path].code.clone(),
                    path: ctx.path.into(),
                    inner: vec![],
                },
                ValidationError {
                    level: ValidationErrorLevel::Error,
                    message: format!("expression type is `{expr_type}`"),
                    span: expr.span(),
                    code: ctx.asts[ctx.path].code.clone(),
                    path: ctx.path.into(),
                    inner: vec![],
                },
            ],
        });
    }
}

use crate::config::ValidationConfig;
use crate::validation::ValidationContext;
use crate::{AstNode, ValidationError, ValidationErrorLevel};

pub(crate) fn check_ident_uniqueness(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let parents: Vec<_> = validation.params["parents"].split(';').collect();
    let ident = &validation.params["ident"];
    let parent_id = node.parent_ids.last().copied().unwrap_or(0);
    for duplicated in ctx.asts[ctx.path]
        .index
        .indexed_nodes
        .get(&node.slice)
        .iter()
        .flat_map(|nodes| *nodes)
        .filter(|node| node.id < parent_id && parents.contains(&node.kind_name.as_str()))
    {
        ctx.errors.push(ValidationError {
            level: ValidationErrorLevel::Error,
            message: "duplicated identifier".into(),
            span: node.span(),
            code: ctx.asts[ctx.path].code.clone(),
            path: ctx.path.into(),
            inner: vec![ValidationError {
                level: ValidationErrorLevel::Info,
                message: "same identifier defined here".into(),
                span: duplicated.child(ident).span(),
                code: ctx.asts[ctx.path].code.clone(),
                path: ctx.path.into(),
                inner: vec![],
            }],
        });
    }
}

pub(crate) fn check_existing_source(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    if node.source(ctx.asts, ctx.path).is_none() {
        ctx.errors.push(ValidationError {
            level: ValidationErrorLevel::Error,
            message: "undefined identifier".into(),
            span: node.span(),
            code: ctx.asts[ctx.path].code.clone(),
            path: ctx.path.into(),
            inner: vec![],
        });
    }
}

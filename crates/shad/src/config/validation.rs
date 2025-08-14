use crate::compilation::ast::AstNode;
use crate::compilation::error::{ValidationError, ValidationMessageLevel};
use crate::compilation::validation::ValidationContext;
use crate::config::ValidationConfig;
use std::str::FromStr;

pub(crate) fn run(ctx: &mut ValidationContext<'_>, validation: &ValidationConfig, node: &AstNode) {
    match validation.name.as_str() {
        "check_number_range" => check_number_range(ctx, validation, node),
        "check_ident_uniqueness" => check_ident_uniqueness(ctx, validation, node),
        "check_existing_source" => check_existing_source(ctx, node),
        "check_expr_type" => check_expr_type(ctx, validation, node),
        "check_import_path" => check_import_path(ctx, node),
        validation_name => unreachable!("undefined `{validation_name}` validation"),
    }
}

fn check_number_range(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let type_ = &validation.params["type"];
    let removed_chars: Vec<_> = validation.params["removed_chars"].chars().collect();
    let slice = node.slice.replace(removed_chars.as_slice(), "");
    let is_invalid_range = match type_.as_str() {
        "i32" => i32::from_str(&slice).is_err(),
        "u32" => u32::from_str(&slice).is_err(),
        "f32" => match f32::from_str(&slice) {
            Ok(value) => value.is_infinite(),
            Err(_) => true,
        },
        _ => unreachable!("undefined `{type_}` number type"),
    };
    if is_invalid_range {
        ctx.errors.push(ValidationError {
            level: ValidationMessageLevel::Error,
            message: format!("out of bound `{type_}` literal"),
            span: node.span(),
            code: ctx.asts[&node.path].code.clone(),
            path: node.path.clone(),
            inner: vec![],
        });
    }
}

fn check_ident_uniqueness(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let parents: Vec<_> = validation.params["parents"].split(';').collect();
    let ident = &validation.params["ident"];
    let parent_id = node.parent_ids.last().copied().unwrap_or(0);
    for duplicated in ctx.asts[&node.path]
        .index
        .indexed_nodes
        .get(&node.slice)
        .iter()
        .flat_map(|nodes| *nodes)
        .filter(|node| node.id < parent_id && parents.contains(&node.kind_name.as_str()))
    {
        ctx.errors.push(ValidationError {
            level: ValidationMessageLevel::Error,
            message: "identifier defined multiple times".into(),
            span: node.span(),
            code: ctx.asts[&node.path].code.clone(),
            path: node.path.clone(),
            inner: vec![
                ValidationError {
                    level: ValidationMessageLevel::Error,
                    message: "duplicated identifier".into(),
                    span: node.span(),
                    code: ctx.asts[&node.path].code.clone(),
                    path: node.path.clone(),
                    inner: vec![],
                },
                ValidationError {
                    level: ValidationMessageLevel::Info,
                    message: "same identifier defined here".into(),
                    span: duplicated.child(ident).span(),
                    code: ctx.asts[&node.path].code.clone(),
                    path: node.path.clone(),
                    inner: vec![],
                },
            ],
        });
    }
}

fn check_existing_source(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    if node.source(ctx.asts).is_none() {
        ctx.errors.push(ValidationError {
            level: ValidationMessageLevel::Error,
            message: "undefined identifier".into(),
            span: node.span(),
            code: ctx.asts[&node.path].code.clone(),
            path: node.path.clone(),
            inner: vec![],
        });
    }
}

fn check_expr_type(ctx: &mut ValidationContext<'_>, validation: &ValidationConfig, node: &AstNode) {
    let expr = node.child(&validation.params["expr"]);
    let expected = node.child(&validation.params["expected"]);
    let Some(expr_type) = expr.type_(ctx.asts) else {
        return;
    };
    let Some(expected_type) = expected.type_(ctx.asts) else {
        return;
    };
    if expr_type != expected_type {
        ctx.errors.push(ValidationError {
            level: ValidationMessageLevel::Error,
            message: "invalid expression type".into(),
            span: expr.span(),
            code: ctx.asts[&node.path].code.clone(),
            path: node.path.clone(),
            inner: vec![
                ValidationError {
                    level: ValidationMessageLevel::Info,
                    message: format!("expected type is `{expected_type}`"),
                    span: expected.span(),
                    code: ctx.asts[&node.path].code.clone(),
                    path: node.path.clone(),
                    inner: vec![],
                },
                ValidationError {
                    level: ValidationMessageLevel::Error,
                    message: format!("expression type is `{expr_type}`"),
                    span: expr.span(),
                    code: ctx.asts[&node.path].code.clone(),
                    path: node.path.clone(),
                    inner: vec![],
                },
            ],
        });
    }
}

fn check_import_path(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    let path = node.import_path(ctx.root_path);
    if !ctx.asts.contains_key(&path) {
        ctx.errors.push(ValidationError {
            level: ValidationMessageLevel::Error,
            message: "imported file not found".into(),
            span: node.span(),
            code: ctx.asts[&node.path].code.clone(),
            path: node.path.clone(),
            inner: vec![ValidationError {
                level: ValidationMessageLevel::Error,
                message: format!("no file found at `{}`", path.display()),
                span: node.span(),
                code: ctx.asts[&node.path].code.clone(),
                path: node.path.clone(),
                inner: vec![],
            }],
        });
    }
}

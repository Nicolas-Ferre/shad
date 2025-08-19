use crate::compilation::ast::AstNode;
use crate::compilation::error::{ValidationError, ValidationMessageLevel};
use crate::compilation::validation::ValidationContext;
use crate::config::ValidationConfig;
use std::str::FromStr;

pub(crate) fn run(ctx: &mut ValidationContext<'_>, validation: &ValidationConfig, node: &AstNode) {
    match validation.name.as_str() {
        "check_number_range" => check_number_range(ctx, validation, node),
        "check_root_index_key_uniqueness" => check_root_index_key_uniqueness(ctx, node),
        "check_existing_source" => check_existing_source(ctx, node),
        "check_expr_type" => check_expr_type(ctx, validation, node),
        "check_missing_return_stmt" => check_missing_return_stmt(ctx, validation, node),
        "check_invalid_return_type" => check_invalid_return_type(ctx, validation, node),
        "check_import_path" => check_import_path(ctx, node),
        "check_invalid_expr_type" => check_invalid_expr_type(ctx, validation, node),
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
        "f32" => f32::from_str(&slice)
            .expect("internal error: invalid `f32` literal")
            .is_infinite(),
        _ => unreachable!("undefined `{type_}` number type"),
    };
    if is_invalid_range {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            node,
            format!("out of bound `{type_}` literal"),
            None,
            vec![],
        ));
    }
}

fn check_root_index_key_uniqueness(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    let key = node.key();
    for other_node in ctx.asts[&node.path].root.children() {
        if other_node.id >= node.id {
            return;
        }
        if other_node.key() == key {
            ctx.errors.push(ValidationError::new(
                ctx,
                ValidationMessageLevel::Error,
                node,
                format!("{key} defined multiple times"),
                Some("duplicated item".into()),
                vec![ValidationError::new(
                    ctx,
                    ValidationMessageLevel::Info,
                    other_node,
                    "same item defined here",
                    None,
                    vec![],
                )],
            ));
        }
    }
}

fn check_existing_source(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    if node.source(ctx.asts).is_none() {
        let source_key = node
            .source_key(ctx.asts)
            .expect("internal error: invalid source config");
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            node,
            "undefined item",
            Some(format!("{source_key} is undefined")),
            vec![],
        ));
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
    let skipped_type = &validation.params["skipped_type"];
    if expr_type != expected_type && &expr_type != skipped_type && &expected_type != skipped_type {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            expr,
            "invalid expression type",
            Some(format!("expression type is `{expr_type}`")),
            vec![ValidationError::new(
                ctx,
                ValidationMessageLevel::Info,
                expected,
                format!("expected type is `{expected_type}`"),
                None,
                vec![],
            )],
        ));
    }
}

fn check_missing_return_stmt(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let expected = node.child(&validation.params["return_type"]);
    let Some(expected_type) = expected.type_(ctx.asts) else {
        return;
    };
    let body = node.child(&validation.params["body"]);
    let last_stmt = body
        .child(&validation.params["body_inner"])
        .children()
        .last();
    let last_stmt_or_default = last_stmt.map_or(body, |stmt| stmt);
    if last_stmt.is_none_or(|stmt| stmt.kind_name != validation.params["return_stmt"])
        && expected_type != validation.params["no_return_type"]
    {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            last_stmt_or_default,
            "missing return statement",
            Some("last statement should be a `return` statement".into()),
            vec![ValidationError::new(
                ctx,
                ValidationMessageLevel::Info,
                expected,
                "the function has a return type",
                None,
                vec![],
            )],
        ));
    }
}

fn check_invalid_return_type(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let no_return_type = &validation.params["no_return_type"];
    let expected = node.child(&validation.params["return_type"]);
    let Some(expected_type) = expected.type_(ctx.asts) else {
        return;
    };
    let body = node.child(&validation.params["body"]);
    let Some(last_stmt) = body
        .child(&validation.params["body_inner"])
        .children()
        .last()
    else {
        return;
    };
    let Some(return_type) = last_stmt.type_(ctx.asts) else {
        return;
    };
    if return_type != expected_type
        && &return_type != no_return_type
        && &expected_type != no_return_type
    {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            last_stmt,
            "invalid returned type",
            Some(format!("returned type is `{return_type}`")),
            vec![ValidationError::new(
                ctx,
                ValidationMessageLevel::Info,
                expected,
                format!("expected type is `{expected_type}`"),
                None,
                vec![],
            )],
        ));
    }
}

fn check_import_path(ctx: &mut ValidationContext<'_>, node: &AstNode) {
    let path = node.import_path(ctx.root_path);
    if !ctx.asts.contains_key(&path) {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            node,
            "imported file not found",
            Some(format!("no file found at `{}`", path.display())),
            vec![],
        ));
    }
}

fn check_invalid_expr_type(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let forbidden_type = &validation.params["type"];
    if node.type_(ctx.asts).as_ref() == Some(forbidden_type) {
        ctx.errors.push(ValidationError::new(
            ctx,
            ValidationMessageLevel::Error,
            node,
            "invalid expression type",
            Some("this function does not return a value".into()),
            vec![],
        ));
    }
}

use crate::compilation::node::{Node, NodeType};
use crate::compilation::validation::ValidationContext;
use crate::language::items;
use crate::language::patterns::Ident;
use crate::ValidationError;

pub(crate) fn check_missing_source(node: &impl Node, ctx: &mut ValidationContext<'_>) {
    if let Some(key) = node.source_key(ctx.index) {
        if node.source(ctx.index).is_none() {
            ctx.errors.push(ValidationError::error(
                ctx,
                node,
                "undefined item",
                Some(&format!("{key} is undefined")),
                &[],
            ));
        }
    }
}

pub(crate) fn check_duplicated_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    let key = item
        .key()
        .expect("internal error: cannot calculate item key");
    for other_item in ctx.roots[&item.path].items.iter() {
        let other_item = other_item.inner();
        if other_item.id < item.id && other_item.key().as_ref() == Some(&key) {
            ctx.errors.push(ValidationError::error(
                ctx,
                item,
                &format!("{key} defined multiple times"),
                Some("duplicated item"),
                &[(other_item, "same item defined here")],
            ));
        }
    }
}

pub(crate) fn check_recursive_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    if items::is_item_recursive(item, ctx.index) {
        ctx.errors.push(ValidationError::error(
            ctx,
            item,
            "item definition with circular dependency",
            Some("this item is directly or indirectly referring to itself"),
            &[],
        ));
    }
}

pub(crate) fn check_invalid_expr_type(
    expected: &dyn Node,
    actual: &dyn Node,
    check_no_return: bool,
    ctx: &mut ValidationContext<'_>,
) {
    if let (Some(expected_type), Some(actual_type)) =
        (expected.type_(ctx.index), actual.type_(ctx.index))
    {
        if (actual_type.is_no_return() || expected_type.is_no_return()) && !check_no_return {
            return;
        }
        if actual_type.are_same(expected_type, ctx.index) == Some(false) {
            let expected_type_name = expected_type.name_or_no_return(ctx.index);
            let actual_type_name = actual_type.name_or_no_return(ctx.index);
            ctx.errors.push(ValidationError::error(
                ctx,
                actual,
                "invalid expression type",
                Some(&format!("expression type is `{actual_type_name}`")),
                &[(
                    expected,
                    &format!("expected type is `{expected_type_name}`"),
                )],
            ));
        }
    }
}

pub(crate) fn check_invalid_const_expr_type(
    expected_type: NodeType<'_>,
    actual: &dyn Node,
    ctx: &mut ValidationContext<'_>,
) {
    if let Some(actual_type) = actual.type_(ctx.index) {
        if actual_type.are_same(expected_type, ctx.index) == Some(false) {
            let expected_type_name = expected_type.name_or_no_return(ctx.index);
            let actual_type_name = actual_type.name_or_no_return(ctx.index);
            ctx.errors.push(ValidationError::error(
                ctx,
                actual,
                "invalid expression type",
                Some(&format!(
                    "expression type is `{actual_type_name}` but expected type is `{expected_type_name}`"
                )),
                &[],
            ));
        }
    }
}

pub(crate) fn check_invalid_const_scope(
    checked: &impl Node,
    const_declaration: &dyn Node,
    ctx: &mut ValidationContext<'_>,
) {
    if let Some(invalid_node) = checked.invalid_constant(ctx.index) {
        ctx.errors.push(ValidationError::error(
            ctx,
            invalid_node,
            "invalid `const` scope",
            Some("cannot be used in a `const` scope"),
            &[(const_declaration, "`const` scope declared here")],
        ));
    }
}

pub(crate) fn check_arg_name(
    arg_name: Option<&Ident>,
    expected_name: &Ident,
    ctx: &mut ValidationContext<'_>,
) {
    if let Some(arg_name) = arg_name {
        if arg_name.slice != expected_name.slice {
            ctx.errors.push(ValidationError::error(
                ctx,
                arg_name,
                "invalid argument name",
                Some("the invalid argument name"),
                &[(expected_name, "expected name")],
            ));
        }
    }
}

pub(crate) fn check_no_return_type(expr: &impl Node, ctx: &mut ValidationContext<'_>) {
    if expr.type_(ctx.index).is_some_and(NodeType::is_no_return) {
        ctx.errors.push(ValidationError::error(
            ctx,
            expr,
            "invalid expression type",
            Some("this function does not return a value"),
            &[],
        ));
    }
}

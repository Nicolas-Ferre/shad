use crate::compilation::node::Node;
use crate::compilation::validation::ValidationContext;
use crate::language::items;
use crate::language::items::type_;
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
        let expected_type_name = type_::name_or_no_return(expected_type);
        let actual_type_name = type_::name_or_no_return(actual_type);
        if (actual_type.is_no_return() || expected_type.is_no_return()) && !check_no_return {
            return;
        }
        if actual_type.source().map(|s| s.id) != expected_type.source().map(|s| s.id) {
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

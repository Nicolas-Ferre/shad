use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::type_;
use crate::ValidationError;

pub(crate) mod binary;
pub(crate) mod chain;
pub(crate) mod constructor;
pub(crate) mod fn_call;
pub(crate) mod simple;
mod transformations;
pub(crate) mod unary;

sequence!(
    #[allow(unused_mut)]
    struct TypedExpr {
        expr: MaybeBinaryExpr,
    }
);

impl NodeConfig for TypedExpr {
    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.expr.is_ref(index)
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.type_(ctx.index).is_some_and(NodeType::is_no_return) {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "invalid expression type",
                Some("this function does not return a value"),
                &[],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.expr.transpile(ctx)
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

use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::type_::NO_RETURN_TYPE;
use crate::ValidationError;

pub(crate) mod binary;
pub(crate) mod chain;
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
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.expr_type(ctx.index).as_deref() == Some(NO_RETURN_TYPE) {
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

fn check_missing_source(node: &impl Node, ctx: &mut ValidationContext<'_>) {
    if let Some(key) = node.source_key(ctx.index) {
        if node.source_from_key(ctx.index, &key).is_none() {
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

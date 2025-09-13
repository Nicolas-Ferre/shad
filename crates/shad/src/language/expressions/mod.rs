use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
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

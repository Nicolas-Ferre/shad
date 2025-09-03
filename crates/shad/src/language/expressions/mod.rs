use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::Expr;
use crate::language::items::fn_::NativeFnItem;
use crate::language::items::type_::NO_RETURN_TYPE;
use crate::ValidationError;
use itertools::Itertools;
use std::any::Any;

pub(crate) mod binary;
pub(crate) mod fn_call;
pub(crate) mod operand;
pub(crate) mod simple;
mod transformations;
pub(crate) mod unary;

sequence!(
    #[allow(unused_mut)]
    struct TypedExpr {
        expr: Expr,
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

fn transpile_fn_call<'a>(
    ctx: &mut TranspilationContext<'_>,
    fn_: &dyn Node,
    args: impl Iterator<Item = &'a impl Node>,
) -> String {
    if let Some(native_fn) = (fn_ as &dyn Any).downcast_ref::<NativeFnItem>() {
        let mut transpilation =
            native_fn.transpilation.slice[1..native_fn.transpilation.slice.len() - 1].to_string();
        for (arg, param) in args.zip(native_fn.signature.params()) {
            transpilation = transpilation.replace(&param.ident.slice, &arg.transpile(ctx));
        }
        transpilation
    } else {
        let fn_id = fn_.id;
        let args = args.map(|arg| arg.transpile(ctx)).join(", ");
        format!("_{fn_id}({args})")
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

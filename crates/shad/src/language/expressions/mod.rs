use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::Expr;
use crate::language::items::fn_::{FnItem, NativeFnItem};
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
    } else if let Some(fn_) = (fn_ as &dyn Any).downcast_ref::<FnItem>() {
        let is_inlined = fn_
            .signature
            .params()
            .any(|param| param.ref_.iter().len() == 1);
        if is_inlined {
            transpile_inlined_fn_call(ctx, fn_, args)
        } else {
            let fn_id = fn_.id;
            let args = args.map(|arg| arg.transpile(ctx)).join(", ");
            format!("_{fn_id}({args})")
        }
    } else {
        unreachable!("unknown function item")
    }
}

fn transpile_inlined_fn_call<'a>(
    ctx: &mut TranspilationContext<'_>,
    fn_: &FnItem,
    args: impl Iterator<Item = &'a impl Node>,
) -> String {
    let old_state = ctx.inline_state;
    ctx.inline_state.is_inlined = true;
    ctx.start_block();
    let return_var_name = if let Some(return_type) = fn_.signature.return_type.iter().next() {
        let return_var_id = ctx.next_node_id();
        let return_var_name = format!("_{return_var_id}");
        let return_type = return_type.type_.transpile(ctx);
        ctx.generated_stmts
            .push(format!("var {return_var_name}: {return_type};"));
        ctx.inline_state.return_var_id = Some(return_var_id);
        Some(return_var_name)
    } else {
        None
    };
    for (param, arg) in fn_.signature.params().zip(args) {
        let transpiled_arg = arg.transpile(ctx);
        if param.ref_.iter().len() == 1 {
            ctx.add_inline_mapping(param.id, transpiled_arg);
        } else {
            let var_id = ctx.next_node_id();
            let var_name = format!("_{var_id}");
            ctx.generated_stmts
                .push(format!("var {var_name} = {transpiled_arg};"));
            ctx.add_inline_mapping(param.id, var_name);
        }
    }
    let inlined_stmts = fn_.body.transpile(ctx);
    ctx.generated_stmts.push(inlined_stmts);
    ctx.end_block();
    ctx.inline_state = old_state;
    return_var_name.unwrap_or_default()
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

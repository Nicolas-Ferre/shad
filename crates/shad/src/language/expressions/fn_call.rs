use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeSourceSearchCriteria, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::Expr;
use crate::language::expressions::check_missing_source;
use crate::language::items::fn_::{FnItem, NativeFnItem};
use crate::language::keywords::{
    CloseParenthesisSymbol, CommaSymbol, DotSymbol, OpenParenthesisSymbol,
};
use crate::language::patterns::Ident;
use crate::language::sources;
use itertools::Itertools;
use std::any::Any;
use std::iter;
use std::rc::Rc;

sequence!(
    struct AssociatedFnCallSuffix {
        dot: DotSymbol,
        #[force_error(true)]
        ident: Ident,
        args_start: OpenParenthesisSymbol,
        args: Repeated<FnArgGroup, 0, 1>,
        args_end: CloseParenthesisSymbol,
    }
);

impl NodeConfig for AssociatedFnCallSuffix {}

sequence!(
    struct FnCallExpr {
        ident: Ident,
        args_start: OpenParenthesisSymbol,
        #[force_error(true)]
        args: Repeated<FnArgGroup, 0, 1>,
        args_end: CloseParenthesisSymbol,
    }
);

impl NodeConfig for FnCallExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        sources::fn_key_from_args(&self.ident, self.args(), index)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::fn_criteria()
    }

    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.source(index)
            .is_some_and(|source| source.is_ref(index))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.source(index)?.expr_type(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let source = self
            .source(ctx.index)
            .expect("internal error: fn call source not found");
        transpile_fn_call(ctx, source, self.args())
    }
}

impl FnCallExpr {
    fn args(&self) -> impl Iterator<Item = &Expr> {
        self.args
            .iter()
            .flat_map(|args| args.args().map(|arg| &**arg))
    }
}

sequence!(
    struct FnArgGroup {
        first_arg: Expr,
        #[force_error(true)]
        other_args: Repeated<FnOtherArg, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
    }
);

impl NodeConfig for FnArgGroup {}

impl FnArgGroup {
    pub(crate) fn args(&self) -> impl Iterator<Item = &Rc<Expr>> {
        iter::once(&self.first_arg).chain(self.other_args.iter().map(|other| &other.arg))
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FnOtherArg {
        comma: CommaSymbol,
        arg: Expr,
    }
);

impl NodeConfig for FnOtherArg {}

pub(crate) fn transpile_fn_call<'a>(
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

pub(crate) fn transpile_inlined_fn_call<'a>(
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
        if param.ref_.iter().len() == 1 && arg.is_ref(ctx.index) {
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

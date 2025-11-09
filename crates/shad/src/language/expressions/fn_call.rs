use crate::compilation::constant::{ConstantContext, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    sequence, GenericArgs, Node, NodeConfig, NodeRef, NodeSource, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::fn_;
use crate::language::items::fn_::{FnItem, NativeFnItem};
use crate::language::keywords::{
    CloseParenthesisSymbol, ColonSymbol, CommaSymbol, OpenParenthesisSymbol,
};
use crate::language::patterns::Ident;
use crate::language::{constants, sources};
use crate::language::{transpilation, validations};
use itertools::Itertools;
use std::any::Any;
use std::iter;
use std::rc::Rc;

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

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        let source = index.search(self, &self.source_key(index)?, sources::fn_criteria())?;
        Some(NodeSource {
            node: NodeRef::Other(source),
            generic_args: vec![],
        })
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.source(index)
            .and_then(|source| source.as_node().is_ref(index))
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.source(index)?.as_node().type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
        if let Some(source) = self.source(ctx.index) {
            let arg_names = self
                .args
                .iter()
                .flat_map(|args| args.args())
                .map(|arg| arg.name.iter().map(|arg| &*arg.ident).next());
            check_arg_names(&source, arg_names, ctx);
        }
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.args()
            .find_map(|arg| arg.invalid_constant(index))
            .or_else(|| (!fn_::is_const(self.source(index)?.as_node())).then_some(self))
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let fn_ = self.source(ctx.index)?.as_node();
        let args = constants::evaluate_fn_args(fn_, self.args(), ctx);
        ctx.start_fn(args);
        let value = fn_.evaluate_constant(ctx);
        ctx.end_fn();
        value
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        let source = self
            .source(ctx.index)
            .expect("internal error: fn call source not found");
        transpile_fn_call(ctx, &source, self.args(), generic_args)
    }
}

impl FnCallExpr {
    fn args(&self) -> impl Iterator<Item = &MaybeBinaryExpr> {
        self.args
            .iter()
            .flat_map(|args| args.args().map(|arg| &*arg.expr))
    }
}

sequence!(
    struct FnArgGroup {
        first_arg: FnArg,
        #[force_error(true)]
        other_args: Repeated<FnOtherArg, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
    }
);

impl NodeConfig for FnArgGroup {}

impl FnArgGroup {
    pub(crate) fn args(&self) -> impl Iterator<Item = &Rc<FnArg>> {
        iter::once(&self.first_arg).chain(self.other_args.iter().map(|other| &other.arg))
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FnOtherArg {
        comma: CommaSymbol,
        arg: FnArg,
    }
);

impl NodeConfig for FnOtherArg {}

sequence!(
    #[allow(unused_mut)]
    struct FnArg {
        name: Repeated<FnArgName, 0, 1>,
        expr: MaybeBinaryExpr,
    }
);

impl NodeConfig for FnArg {
    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.expr.type_(index)
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.expr.evaluate_constant(ctx)
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        self.expr.transpile(ctx, generic_args)
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FnArgName {
        ident: Ident,
        colon: ColonSymbol,
    }
);

impl NodeConfig for FnArgName {}

pub(crate) fn check_arg_names<'a>(
    fn_: &NodeSource<'_>,
    arg_names: impl Iterator<Item = Option<&'a Ident>>,
    ctx: &mut ValidationContext<'_>,
) {
    for (arg_name, param) in arg_names.zip(fn_::signature(fn_.as_node()).params()) {
        validations::check_arg_name(arg_name, &param.ident, ctx);
    }
}

pub(crate) fn transpile_fn_call<'a>(
    ctx: &mut TranspilationContext<'_>,
    fn_: &NodeSource<'_>,
    args: impl Iterator<Item = &'a impl Node>,
    generic_args: &GenericArgs<'_>,
) -> String {
    let node = fn_.as_node() as &dyn Any;
    if let Some(native_fn) = node.downcast_ref::<NativeFnItem>() {
        let params = native_fn.signature.params().map(|p| &p.ident.slice);
        let args = args.map(|arg| arg.transpile(ctx, generic_args));
        transpilation::resolve_placeholders(native_fn.transpilation.as_str(), params, args)
    } else if let Some(fn_) = node.downcast_ref::<FnItem>() {
        if fn_.is_inlined(ctx.index) {
            transpile_inlined_fn_call(ctx, fn_, args, generic_args)
        } else {
            let fn_id = fn_.id;
            let args = args.map(|arg| arg.transpile(ctx, generic_args)).join(", ");
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
    generic_args: &GenericArgs<'_>,
) -> String {
    let old_state = ctx.inline_state.clone();
    ctx.inline_state.is_inlined = true;
    ctx.start_block();
    let return_var_name = if fn_.is_ref(ctx.index) == Some(true) {
        ctx.inline_state.is_returning_ref = true;
        None
    } else if let Some(return_type) = fn_.signature.return_type.iter().next() {
        let return_var_id = ctx.next_node_id();
        let return_var_name = format!("_{return_var_id}");
        let return_type = return_type.type_.transpile(ctx, generic_args);
        ctx.generated_stmts
            .push(format!("var {return_var_name}: {return_type};"));
        ctx.inline_state.return_var_id = Some(return_var_id);
        ctx.inline_state.is_returning_ref = false;
        Some(return_var_name)
    } else {
        None
    };
    for (param, arg) in fn_.signature.params().zip(args) {
        let transpiled_arg = arg.transpile(ctx, generic_args);
        if param.is_ref(ctx.index) == Some(true) && arg.is_ref(ctx.index) == Some(true) {
            ctx.add_inline_mapping(param.id, transpiled_arg);
        } else {
            let var_id = ctx.next_node_id();
            let var_name = format!("_{var_id}");
            ctx.generated_stmts
                .push(format!("var {var_name} = {transpiled_arg};"));
            ctx.add_inline_mapping(param.id, var_name);
        }
    }
    let inlined_stmts = fn_.body.transpile(ctx, &vec![]);
    ctx.generated_stmts.push(inlined_stmts);
    ctx.end_block();
    let returned_ref = ctx.inline_state.returned_ref.take();
    ctx.inline_state = old_state;
    returned_ref.or(return_var_name).unwrap_or_default()
}

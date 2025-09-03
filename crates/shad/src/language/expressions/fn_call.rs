use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeSourceSearchCriteria, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::Expr;
use crate::language::expressions::{check_missing_source, transpile_fn_call};
use crate::language::keywords::{
    CloseParenthesisSymbol, CommaSymbol, DotSymbol, OpenParenthesisSymbol,
};
use crate::language::patterns::Ident;
use crate::language::sources;
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

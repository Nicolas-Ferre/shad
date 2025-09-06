use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, transform, Node, NodeConfig, NodeSourceSearchCriteria, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::expressions::fn_call::{
    transpile_fn_call, AssociatedFnCallSuffix, FnCallExpr,
};
use crate::language::expressions::simple::{FalseExpr, ParenthesizedExpr, TrueExpr, VarIdentExpr};
use crate::language::expressions::unary::UnaryExpr;
use crate::language::expressions::{check_missing_source, transformations};
use crate::language::patterns::{F32Literal, I32Literal, U32Literal};
use crate::language::sources;
use std::iter;

transform!(
    ChainExpr,
    ParsedChainExpr,
    TransformedChainExpr,
    transformations::transform_chain_expr
);

sequence!(
    struct ParsedChainExpr {
        expr: ChainPrefix,
        #[force_error(true)]
        call_suffix: Repeated<AssociatedFnCallSuffix, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedChainExpr {
    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.expr.is_ref(index)
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn validate(&self, _ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.call_suffix.iter().len() == 0);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.expr.transpile(ctx)
    }
}

sequence!(
    #[allow(unused_mut)]
    struct TransformedChainExpr {
        expr: MaybeBinaryExpr,
        call_suffix: Repeated<AssociatedFnCallSuffix, 0, 1>,
    }
);

impl NodeConfig for TransformedChainExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        let suffix = &self.call_suffix.iter().next()?;
        sources::fn_key_from_args(&suffix.ident, self.args(suffix), index)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::fn_criteria()
    }

    fn is_ref(&self, index: &NodeIndex) -> bool {
        if self.call_suffix.iter().next().is_some() {
            self.source(index)
                .is_some_and(|source| source.is_ref(index))
        } else {
            self.expr.is_ref(index)
        }
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        if self.call_suffix.iter().next().is_some() {
            self.source(index)?.expr_type(index)
        } else {
            self.expr.expr_type(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.call_suffix.iter().len() <= 1);
        check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        if let Some(suffix) = &self.call_suffix.iter().next() {
            let source = self
                .source(ctx.index)
                .expect("internal error: fn call source not found");
            transpile_fn_call(ctx, source, self.args(suffix))
        } else {
            self.expr.transpile(ctx)
        }
    }
}

impl TransformedChainExpr {
    fn args<'a>(
        &'a self,
        suffix: &'a AssociatedFnCallSuffix,
    ) -> impl Iterator<Item = &'a MaybeBinaryExpr> {
        iter::once(&*self.expr).chain(
            suffix
                .args
                .iter()
                .flat_map(|args| args.args().map(|arg| &**arg)),
        )
    }
}

choice!(
    // TODO: remove unary exprs
    #[allow(clippy::large_enum_variant)]
    enum ChainPrefix {
        True(TrueExpr),
        False(FalseExpr),
        F32(F32Literal),
        U32(U32Literal),
        I32(I32Literal),
        FnCall(FnCallExpr),
        Var(VarIdentExpr),
        Unary(UnaryExpr),
        Parenthesized(ParenthesizedExpr),
    }
);

impl NodeConfig for ChainPrefix {
    fn is_ref(&self, index: &NodeIndex) -> bool {
        match self {
            Self::True(_)
            | Self::False(_)
            | Self::F32(_)
            | Self::U32(_)
            | Self::I32(_)
            | Self::Parenthesized(_) => false,
            Self::Var(_) => true,
            Self::FnCall(child) => child.is_ref(index),
            Self::Unary(child) => child.is_ref(index),
        }
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        match self {
            Self::True(child) => child.expr_type(index),
            Self::False(child) => child.expr_type(index),
            Self::F32(child) => child.expr_type(index),
            Self::U32(child) => child.expr_type(index),
            Self::I32(child) => child.expr_type(index),
            Self::FnCall(child) => child.expr_type(index),
            Self::Var(child) => child.expr_type(index),
            Self::Unary(child) => child.expr_type(index),
            Self::Parenthesized(child) => child.expr_type(index),
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        match self {
            Self::True(child) => child.transpile(ctx),
            Self::False(child) => child.transpile(ctx),
            Self::F32(child) => child.transpile(ctx),
            Self::U32(child) => child.transpile(ctx),
            Self::I32(child) => child.transpile(ctx),
            Self::FnCall(child) => child.transpile(ctx),
            Self::Var(child) => child.transpile(ctx),
            Self::Unary(child) => child.transpile(ctx),
            Self::Parenthesized(child) => child.transpile(ctx),
        }
    }
}

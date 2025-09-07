use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, transform, Node, NodeConfig, NodeSourceSearchCriteria, NodeType, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::expressions::fn_call::{
    transpile_fn_call, AssociatedFnCallSuffix, FnCallExpr,
};
use crate::language::expressions::simple::{BoolLiteral, ParenthesizedExpr, VarIdentExpr};
use crate::language::expressions::transformations;
use crate::language::expressions::unary::UnaryExpr;
use crate::language::patterns::{F32Literal, I32Literal, U32Literal};
use crate::language::sources;
use crate::language::sources::check_missing_source;
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

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
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

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        if self.call_suffix.iter().next().is_some() {
            self.source(index)?.type_(index)
        } else {
            self.expr.type_(index)
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
    #[allow(clippy::large_enum_variant)]
    enum ChainPrefix {
        Bool(BoolLiteral),
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
            Self::Bool(_) | Self::F32(_) | Self::U32(_) | Self::I32(_) | Self::Parenthesized(_) => {
                false
            }
            Self::Var(_) => true,
            Self::FnCall(child) => child.is_ref(index),
            Self::Unary(child) => child.is_ref(index),
        }
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        match self {
            Self::Bool(child) => child.type_(index),
            Self::F32(child) => child.type_(index),
            Self::U32(child) => child.type_(index),
            Self::I32(child) => child.type_(index),
            Self::FnCall(child) => child.type_(index),
            Self::Var(child) => child.type_(index),
            Self::Unary(child) => child.type_(index),
            Self::Parenthesized(child) => child.type_(index),
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        match self {
            Self::Bool(child) => child.transpile(ctx),
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

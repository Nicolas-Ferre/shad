use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeSourceSearchCriteria};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::{check_missing_source, Expr};
use crate::language::keywords::{
    CloseParenthesisSymbol, FalseKeyword, OpenParenthesisSymbol, TrueKeyword,
};
use crate::language::patterns::Ident;
use crate::language::sources;

sequence!(
    #[allow(unused_mut)]
    struct TrueExpr {
        value: TrueKeyword,
    }
);

impl NodeConfig for TrueExpr {
    fn expr_type(&self, _index: &NodeIndex) -> Option<String> {
        Some("bool".into())
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        "u32(true)".into()
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FalseExpr {
        value: FalseKeyword,
    }
);

impl NodeConfig for FalseExpr {
    fn expr_type(&self, _index: &NodeIndex) -> Option<String> {
        Some("bool".into())
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        "u32(false)".into()
    }
}

sequence!(
    #[allow(unused_mut)]
    struct VarIdentExpr {
        ident: Ident,
    }
);

impl NodeConfig for VarIdentExpr {
    fn source_key(&self, _index: &NodeIndex) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::variable_criteria()
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.source(index)?.expr_type(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let source_id = self
            .source(ctx.index)
            .expect("internal error: var ident source not found")
            .id;
        if let Some(mapping) = ctx.inline_mapping(source_id) {
            mapping.to_string()
        } else {
            format!("_{source_id}")
        }
    }
}

sequence!(
    struct ParenthesizedExpr {
        open: OpenParenthesisSymbol,
        #[force_error(true)]
        expr: Expr,
        close: CloseParenthesisSymbol,
    }
);

impl NodeConfig for ParenthesizedExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        self.expr.source_key(index)
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        format!("({expr})")
    }
}

use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, Node, NodeConfig, NodeSourceSearchCriteria, NodeType,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::MaybeBinaryExpr;
use crate::language::keywords::{
    CloseParenthesisSymbol, FalseKeyword, OpenParenthesisSymbol, TrueKeyword,
};
use crate::language::patterns::Ident;
use crate::language::sources;
use crate::language::validations;

choice!(
    enum BoolLiteral {
        True(TrueKeyword),
        False(FalseKeyword),
    }
);

impl NodeConfig for BoolLiteral {
    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::type_criteria()
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        let source = index
            .search(self, "`bool` type")
            .expect("internal error: `bool` type not found");
        Some(NodeType::Source(source))
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let value = match self {
            Self::True(_) => "true",
            Self::False(_) => "false",
        };
        format!("u32({value})")
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

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::variable_criteria()
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.source(index)?.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
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
        expr: MaybeBinaryExpr,
        close: CloseParenthesisSymbol,
    }
);

impl NodeConfig for ParenthesizedExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        self.expr.source_key(index)
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?)
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        format!("({expr})")
    }
}

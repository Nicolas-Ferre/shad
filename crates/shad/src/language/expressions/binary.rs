use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, transform, Node, NodeConfig, NodeSourceSearchCriteria, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::chain::ChainExpr;
use crate::language::expressions::fn_call::transpile_fn_call;
use crate::language::expressions::{check_missing_source, transformations};
use crate::language::keywords::{
    AndSymbol, CloseAngleBracketSymbol, DoubleEqSymbol, GreaterEqSymbol, HyphenSymbol,
    LessEqSymbol, NotEqSymbol, OpenAngleBracketSymbol, OrSymbol, PercentSymbol, PlusSymbol,
    SlashSymbol, StarSymbol,
};
use crate::language::sources;

transform!(
    MaybeBinaryExpr,
    ParsedMaybeBinaryExpr,
    BinaryExpr,
    transformations::transform_binary_expr
);

sequence!(
    struct ParsedMaybeBinaryExpr {
        left: ChainExpr,
        #[force_error(true)]
        right: Repeated<ParsedBinaryRight, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedMaybeBinaryExpr {
    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.left.is_ref(index)
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.left.expr_type(index)
    }

    fn validate(&self, _ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.right.iter().len() == 0);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.left.transpile(ctx)
    }
}

sequence!(
    #[allow(unused_mut)]
    struct BinaryExpr {
        left: MaybeBinaryExpr,
        operator: BinaryOperator,
        right: MaybeBinaryExpr,
    }
);

impl NodeConfig for BinaryExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        let name = match &*self.operator {
            BinaryOperator::Add(_) => "__add__",
            BinaryOperator::Sub(_) => "__sub__",
            BinaryOperator::Mul(_) => "__mul__",
            BinaryOperator::Div(_) => "__div__",
            BinaryOperator::Mod(_) => "__mod__",
            BinaryOperator::LessEq(_) => "__le__",
            BinaryOperator::GreaterEq(_) => "__ge__",
            BinaryOperator::Less(_) => "__lt__",
            BinaryOperator::Greater(_) => "__gt__",
            BinaryOperator::Eq(_) => "__eq__",
            BinaryOperator::NotEq(_) => "__ne__",
            BinaryOperator::And(_) => "__and__",
            BinaryOperator::Or(_) => "__or__",
        };
        Some(sources::fn_key_from_operator(
            name,
            [self.left.expr_type(index)?, self.right.expr_type(index)?],
        ))
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
        transpile_fn_call(ctx, source, [&*self.left, &*self.right].into_iter())
    }
}

sequence!(
    struct ParsedBinaryRight {
        operator: BinaryOperator,
        #[force_error(true)]
        operand: ChainExpr,
    }
);

impl NodeConfig for ParsedBinaryRight {}

choice!(
    enum BinaryOperator {
        Add(PlusSymbol),
        Sub(HyphenSymbol),
        Mul(StarSymbol),
        Div(SlashSymbol),
        Mod(PercentSymbol),
        LessEq(LessEqSymbol),
        GreaterEq(GreaterEqSymbol),
        Less(OpenAngleBracketSymbol),
        Greater(CloseAngleBracketSymbol),
        Eq(DoubleEqSymbol),
        NotEq(NotEqSymbol),
        And(AndSymbol),
        Or(OrSymbol),
    }
);

impl NodeConfig for BinaryOperator {}

use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, Node, NodeConfig, NodeSourceSearchCriteria};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::chain::ChainExpr;
use crate::language::expressions::check_missing_source;
use crate::language::expressions::fn_call::transpile_fn_call;
use crate::language::keywords::{ExclamationSymbol, HyphenSymbol};
use crate::language::sources;
use std::iter;

sequence!(
    struct UnaryExpr {
        operator: UnaryOperator,
        #[force_error(true)]
        operand: ChainExpr,
    }
);

impl NodeConfig for UnaryExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        let fn_name = match &*self.operator {
            UnaryOperator::Sub(_) => "__neg__",
            UnaryOperator::Not(_) => "__not__",
        };
        let arg_type = self.operand.expr_type(index)?;
        Some(sources::fn_key_from_operator(fn_name, [arg_type]))
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
        transpile_fn_call(ctx, source, iter::once(&*self.operand))
    }
}

choice!(
    enum UnaryOperator {
        Sub(HyphenSymbol),
        Not(ExclamationSymbol),
    }
);

impl NodeConfig for UnaryOperator {}

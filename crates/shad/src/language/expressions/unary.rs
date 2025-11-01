use crate::compilation::constant::{ConstantContext, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, Node, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::chain::ChainExpr;
use crate::language::expressions::fn_call::transpile_fn_call;
use crate::language::items::fn_;
use crate::language::keywords::{ExclamationSymbol, HyphenSymbol};
use crate::language::validations;
use crate::language::{constants, sources};
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
        let arg_type = self.operand.type_(index)?;
        Some(sources::fn_key_from_operator(fn_name, [arg_type]))
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?, sources::fn_criteria())
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.source(index).and_then(|source| source.is_ref(index))
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.source(index)?.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        (!fn_::is_const(self.source(index)?))
            .then_some(self as _)
            .or_else(|| self.operand.invalid_constant(index))
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let args =
            constants::evaluate_fn_args(self.source(ctx.index)?, iter::once(&*self.operand), ctx);
        ctx.start_fn(args);
        let value = self.source(ctx.index)?.evaluate_constant(ctx);
        ctx.end_fn();
        value
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

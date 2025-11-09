use crate::compilation::constant::{ConstantContext, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, GenericArgs, Node, NodeConfig, NodeRef, NodeSource,
};
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

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        let source = index.search(self, &self.source_key(index)?, sources::fn_criteria())?;
        Some(NodeSource {
            node: NodeRef::Other(source),
            generic_args: vec![],
        })
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.source(index)
            .and_then(|source| source.node().is_ref(index))
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.source(index)?.node().type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        (!fn_::is_const(self.source(index)?.node()))
            .then_some(self as _)
            .or_else(|| self.operand.invalid_constant(index))
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let fn_ = self.source(ctx.index)?.node();
        let args = constants::evaluate_fn_args(fn_, iter::once(&*self.operand), ctx);
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
        let fn_ = self
            .source(ctx.index)
            .expect("internal error: fn call source not found");
        transpile_fn_call(ctx, &fn_, iter::once(&*self.operand), generic_args)
    }
}

choice!(
    enum UnaryOperator {
        Sub(HyphenSymbol),
        Not(ExclamationSymbol),
    }
);

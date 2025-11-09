use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, transform, GenericArgs, Node, NodeConfig, NodeRef, NodeSource, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::chain::ChainExpr;
use crate::language::expressions::fn_call::transpile_fn_call;
use crate::language::items::fn_;
use crate::language::keywords::{
    AndSymbol, CloseAngleBracketSymbol, DoubleEqSymbol, GreaterEqSymbol, HyphenSymbol,
    LessEqSymbol, NotEqSymbol, OpenAngleBracketSymbol, OrSymbol, PercentSymbol, PlusSymbol,
    SlashSymbol, StarSymbol,
};
use crate::language::transformations;
use crate::language::validations;
use crate::language::{constants, sources};

transform!(
    MaybeBinaryExpr,
    ParsedMaybeBinaryExpr,
    BinaryExpr,
    transformations::transform_binary_expr
);

impl MaybeBinaryExpr {
    pub(crate) fn parse_const_i32(&self, index: &NodeIndex) -> i32 {
        let mut ctx = ConstantContext::new(index);
        if let Some(ConstantValue {
            data: ConstantData::I32(value),
            ..
        }) = self.evaluate_constant(&mut ctx)
        {
            value
        } else {
            unreachable!("expression should be a constant `i32`");
        }
    }

    pub(crate) fn parse_const_u32(&self, index: &NodeIndex) -> u32 {
        let mut ctx = ConstantContext::new(index);
        if let Some(ConstantValue {
            data: ConstantData::U32(value),
            ..
        }) = self.evaluate_constant(&mut ctx)
        {
            value
        } else {
            unreachable!("expression should be a constant `u32`");
        }
    }
}

sequence!(
    struct ParsedMaybeBinaryExpr {
        left: ChainExpr,
        #[force_error(true)]
        right: Repeated<ParsedBinaryRight, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedMaybeBinaryExpr {
    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.left.is_ref(index)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.left.type_(index)
    }

    fn validate(&self, _ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.right.iter().len() == 0);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.left.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.left.evaluate_constant(ctx)
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        self.left.transpile(ctx, generic_args)
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
            [self.left.type_(index)?, self.right.type_(index)?],
        ))
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
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        (!fn_::is_const(self.source(index)?.as_node()))
            .then_some(self as _)
            .or_else(|| self.left.invalid_constant(index))
            .or_else(|| self.right.invalid_constant(index))
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let fn_ = self.source(ctx.index)?.as_node();
        let args = constants::evaluate_fn_args(fn_, [&*self.left, &*self.right].into_iter(), ctx);
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
        transpile_fn_call(
            ctx,
            &source,
            [&*self.left, &*self.right].into_iter(),
            generic_args,
        )
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

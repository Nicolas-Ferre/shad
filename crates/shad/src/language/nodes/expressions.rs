use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, transform, Node, NodeConfig, NodeSourceSearchCriteria, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::nodes::items::{NativeFnItem, NO_RETURN_TYPE};
use crate::language::nodes::terminals::{
    AndSymbol, CloseAngleBracketSymbol, CloseParenthesisSymbol, CommaSymbol, DotSymbol,
    DoubleEqSymbol, ExclamationSymbol, F32Literal, FalseKeyword, GreaterEqSymbol, HyphenSymbol,
    I32Literal, Ident, LessEqSymbol, NotEqSymbol, OpenAngleBracketSymbol, OpenParenthesisSymbol,
    OrSymbol, PercentSymbol, PlusSymbol, SlashSymbol, StarSymbol, TrueKeyword, U32Literal,
};
use crate::language::{sources, transformations};
use crate::ValidationError;
use itertools::Itertools;
use std::any::Any;
use std::iter;
use std::rc::Rc;

sequence!(
    #[allow(unused_mut)]
    struct TypedExpr {
        expr: Expr,
    }
);

impl NodeConfig for TypedExpr {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.expr_type(ctx.index).as_deref() == Some(NO_RETURN_TYPE) {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "invalid expression type",
                Some("this function does not return a value"),
                &[],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.expr.transpile(ctx)
    }
}

transform!(
    Expr,
    ParsedExpr,
    TransformedExpr,
    transformations::transform_expr
);

impl Expr {
    pub(crate) fn is_fn_call(&self) -> bool {
        match self {
            Self::Parsed(child) => child.is_fn_call(),
            Self::Transformed(child) => child.is_fn_call(),
        }
    }
}

sequence!(
    struct ParsedExpr {
        left: BinaryOperand,
        #[force_error(true)]
        right: Repeated<BinaryRight, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedExpr {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        debug_assert!(self.right.iter().len() == 0);
        self.left.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        debug_assert!(self.right.iter().len() == 0);
        self.left.transpile(ctx)
    }
}

impl ParsedExpr {
    pub(crate) fn is_fn_call(&self) -> bool {
        self.left.is_fn_call()
    }
}

sequence!(
    #[allow(unused_mut)]
    struct TransformedExpr {
        left: Expr,
        operator: BinaryOperator,
        right: Expr,
    }
);

impl NodeConfig for TransformedExpr {
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

impl TransformedExpr {
    #[allow(clippy::unused_self)]
    pub(crate) fn is_fn_call(&self) -> bool {
        false
    }
}

sequence!(
    struct BinaryRight {
        operator: BinaryOperator,
        #[force_error(true)]
        operand: BinaryOperand,
    }
);

impl NodeConfig for BinaryRight {}

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

transform!(
    BinaryOperand,
    ParsedBinaryOperand,
    TransformedBinaryOperand,
    transformations::transform_binary_operand
);

impl BinaryOperand {
    pub(crate) fn is_fn_call(&self) -> bool {
        match self {
            Self::Parsed(child) => child.is_fn_call(),
            Self::Transformed(child) => child.is_fn_call(),
        }
    }
}

sequence!(
    struct ParsedBinaryOperand {
        expr: BinaryOperandPrefix,
        #[force_error(true)]
        call_suffix: Repeated<AssociatedFnCallSuffix, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedBinaryOperand {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        debug_assert!(self.call_suffix.iter().len() == 0);
        self.expr.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        debug_assert!(self.call_suffix.iter().len() == 0);
        self.expr.transpile(ctx)
    }
}

impl ParsedBinaryOperand {
    pub(crate) fn is_fn_call(&self) -> bool {
        self.expr.is_fn_call()
    }
}

sequence!(
    #[allow(unused_mut)]
    struct TransformedBinaryOperand {
        expr: Expr,
        call_suffix: Repeated<AssociatedFnCallSuffix, 0, 1>,
    }
);

impl NodeConfig for TransformedBinaryOperand {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        debug_assert!(self.call_suffix.iter().len() <= 1);
        let suffix = &self.call_suffix.iter().next()?;
        sources::fn_key_from_args(&suffix.ident, self.args(suffix), index)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::fn_criteria()
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        debug_assert!(self.call_suffix.iter().len() <= 1);
        if self.call_suffix.iter().next().is_some() {
            self.source(index)?.expr_type(index)
        } else {
            self.expr.expr_type(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        debug_assert!(self.call_suffix.iter().len() <= 1);
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

impl TransformedBinaryOperand {
    pub(crate) fn is_fn_call(&self) -> bool {
        self.call_suffix.iter().len() == 1 || self.expr.is_fn_call()
    }

    fn args<'a>(&'a self, suffix: &'a AssociatedFnCallSuffix) -> impl Iterator<Item = &'a Expr> {
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
    enum BinaryOperandPrefix {
        True(TrueExpr),
        False(FalseExpr),
        F32(F32Literal),
        U32(U32Literal),
        I32(I32Literal),
        FnCall(FnCallExpr),
        Var(VarIdentExpr),
        NegUnary(NegUnaryExpr),
        NotUnary(NotUnaryExpr),
        Parenthesized(ParenthesizedExpr),
    }
);

impl NodeConfig for BinaryOperandPrefix {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        match self {
            Self::True(child) => child.expr_type(index),
            Self::False(child) => child.expr_type(index),
            Self::F32(child) => child.expr_type(index),
            Self::U32(child) => child.expr_type(index),
            Self::I32(child) => child.expr_type(index),
            Self::FnCall(child) => child.expr_type(index),
            Self::Var(child) => child.expr_type(index),
            Self::NegUnary(child) => child.expr_type(index),
            Self::NotUnary(child) => child.expr_type(index),
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
            Self::NegUnary(child) => child.transpile(ctx),
            Self::NotUnary(child) => child.transpile(ctx),
            Self::Parenthesized(child) => child.transpile(ctx),
        }
    }
}

impl BinaryOperandPrefix {
    pub(crate) fn is_fn_call(&self) -> bool {
        match self {
            Self::FnCall(_) => true,
            Self::True(_)
            | Self::False(_)
            | Self::F32(_)
            | Self::U32(_)
            | Self::I32(_)
            | Self::Var(_)
            | Self::NegUnary(_)
            | Self::NotUnary(_)
            | Self::Parenthesized(_) => false,
        }
    }
}

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
        format!("_{source_id}")
    }
}

sequence!(
    struct NegUnaryExpr {
        minus: HyphenSymbol,
        #[force_error(true)]
        operand: BinaryOperand,
    }
);

impl NodeConfig for NegUnaryExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        let arg_type = self.operand.expr_type(index)?;
        Some(sources::fn_key_from_operator("__neg__", [arg_type]))
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
        transpile_fn_call(ctx, source, iter::once(&*self.operand))
    }
}

sequence!(
    struct NotUnaryExpr {
        not: ExclamationSymbol,
        #[force_error(true)]
        operand: BinaryOperand,
    }
);

impl NodeConfig for NotUnaryExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        let arg_type = self.operand.expr_type(index)?;
        Some(sources::fn_key_from_operator("__not__", [arg_type]))
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
        transpile_fn_call(ctx, source, iter::once(&*self.operand))
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

fn transpile_fn_call<'a>(
    ctx: &mut TranspilationContext<'_>,
    fn_: &dyn Node,
    args: impl Iterator<Item = &'a impl Node>,
) -> String {
    if let Some(native_fn) = (fn_ as &dyn Any).downcast_ref::<NativeFnItem>() {
        let mut transpilation =
            native_fn.transpilation.slice[1..native_fn.transpilation.slice.len() - 1].to_string();
        for (arg, param) in args.zip(native_fn.params()) {
            transpilation = transpilation.replace(&param.ident.slice, &arg.transpile(ctx));
        }
        transpilation
    } else {
        let fn_id = fn_.id;
        let args = args.map(|arg| arg.transpile(ctx)).join(", ");
        format!("_{fn_id}({args})")
    }
}

fn check_missing_source(node: &impl Node, ctx: &mut ValidationContext<'_>) {
    if let Some(key) = node.source_key(ctx.index) {
        if node.source_from_key(ctx.index, &key).is_none() {
            ctx.errors.push(ValidationError::error(
                ctx,
                node,
                "undefined item",
                Some(&format!("{key} is undefined")),
                &[],
            ));
        }
    }
}

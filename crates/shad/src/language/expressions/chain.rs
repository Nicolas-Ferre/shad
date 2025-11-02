use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, transform, Node, NodeConfig, NodeType, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::expressions::constructor::ConstructorExpr;
use crate::language::expressions::fn_call::{
    check_arg_names, transpile_fn_call, FnArgGroup, FnCallExpr,
};
use crate::language::expressions::simple::{
    FalseLiteral, ParenthesizedExpr, TrueLiteral, TypeOperationExpr, VarIdentExpr,
};
use crate::language::expressions::unary::UnaryExpr;
use crate::language::items::fn_;
use crate::language::keywords::{CloseParenthesisSymbol, DotSymbol, OpenParenthesisSymbol};
use crate::language::patterns::{F32Literal, I32Literal, Ident, U32Literal};
use crate::language::transformations;
use crate::language::validations;
use crate::language::{constants, sources};
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
        suffix: Repeated<ChainSuffix, 0, { usize::MAX }>,
    }
);

impl NodeConfig for ParsedChainExpr {
    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.expr.is_ref(index)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn validate(&self, _ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.suffix.iter().len() == 0);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.expr.evaluate_constant(ctx)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.expr.transpile(ctx)
    }
}

sequence!(
    #[allow(unused_mut)]
    struct TransformedChainExpr {
        expr: MaybeBinaryExpr,
        suffix: Repeated<ChainSuffix, 0, 1>,
    }
);

impl NodeConfig for TransformedChainExpr {
    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        match &**self.suffix.iter().next()? {
            ChainSuffix::FnCall(suffix) => {
                sources::fn_key_from_args(&suffix.ident, self.args(suffix), index)
            }
            ChainSuffix::StructField(suffix) => {
                let prefix_type_key = self.expr.type_(index)?.source()?.item.key()?;
                let field_name = &suffix.ident.slice;
                Some(format!("`{field_name}` field of {prefix_type_key}"))
            }
        }
    }

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        match &**self.suffix.iter().next()? {
            ChainSuffix::FnCall(_) => {
                index.search(self, &self.source_key(index)?, sources::fn_criteria())
            }
            ChainSuffix::StructField(suffix) => {
                let expr_type = self.expr.type_(index)?;
                let type_ = expr_type.source()?;
                let field = type_.item.field(&suffix.ident.slice)?;
                (field.is_public() || field.path == self.path).then_some(field)
            }
        }
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        if self.suffix.iter().next().is_some() {
            self.source(index).and_then(|source| source.is_ref(index))
        } else {
            self.expr.is_ref(index)
        }
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        if self.suffix.iter().next().is_some() {
            self.source(index)?.type_(index)
        } else {
            self.expr.type_(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        debug_assert!(self.suffix.iter().len() <= 1);
        validations::check_missing_source(self, ctx);
        if let Some(suffix) = self.suffix.iter().next() {
            match &**suffix {
                ChainSuffix::FnCall(suffix) => {
                    if let Some(source) = self.source(ctx.index) {
                        let arg_names = iter::once(None).chain(
                            suffix
                                .args
                                .iter()
                                .flat_map(|args| args.args())
                                .map(|arg| arg.name.iter().map(|arg| &*arg.ident).next()),
                        );
                        check_arg_names(source, arg_names, ctx);
                    }
                }
                ChainSuffix::StructField(_) => {}
            }
        }
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index).or_else(|| {
            if let Some(suffix) = self.suffix.iter().next() {
                match &**suffix {
                    ChainSuffix::FnCall(suffix) => self
                        .args(suffix)
                        .find_map(|arg| arg.invalid_constant(index))
                        .or_else(|| (!fn_::is_const(self.source(index)?)).then_some(self)),
                    ChainSuffix::StructField(_) => None,
                }
            } else {
                None
            }
        })
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        if let Some(suffix) = self.suffix.iter().next() {
            match &**suffix {
                ChainSuffix::FnCall(suffix) => {
                    let args = constants::evaluate_fn_args(
                        self.source(ctx.index)?,
                        self.args(suffix),
                        ctx,
                    );
                    ctx.start_fn(args);
                    let value = self.source(ctx.index)?.evaluate_constant(ctx);
                    ctx.end_fn();
                    value
                }
                ChainSuffix::StructField(suffix) => {
                    let prefix = self.expr.evaluate_constant(ctx)?;
                    match prefix.data {
                        ConstantData::StructFields(fields) => Some(
                            fields
                                .iter()
                                .find(|field| field.name == suffix.ident.slice)?
                                .value
                                .clone(),
                        ),
                        ConstantData::F32(_)
                        | ConstantData::I32(_)
                        | ConstantData::U32(_)
                        | ConstantData::Bool(_) => {
                            unreachable!("const field used on non-struct value")
                        }
                    }
                }
            }
        } else {
            self.expr.evaluate_constant(ctx)
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        match self.suffix.iter().next().map(|s| &**s) {
            Some(ChainSuffix::FnCall(suffix)) => {
                let source = self
                    .source(ctx.index)
                    .expect("internal error: fn call source not found");
                transpile_fn_call(ctx, source, self.args(suffix))
            }
            Some(ChainSuffix::StructField(suffix)) => {
                let expr_type = self
                    .expr
                    .type_(ctx.index)
                    .expect("internal error: chain prefix type not found");
                let type_ = expr_type
                    .source()
                    .expect("internal error: invalid chain prefix");
                let prefix = self.expr.transpile(ctx);
                let suffix = type_.item.transpiled_field_name(&suffix.ident.slice);
                format!("{prefix}.{suffix}")
            }
            None => self.expr.transpile(ctx),
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
                .flat_map(|args| args.args().map(|arg| &*arg.expr)),
        )
    }
}

choice!(
    #[allow(clippy::large_enum_variant)]
    enum ChainPrefix {
        True(TrueLiteral),
        False(FalseLiteral),
        F32(F32Literal),
        U32(U32Literal),
        I32(I32Literal),
        FnCall(FnCallExpr),
        Constructor(ConstructorExpr),
        Var(VarIdentExpr),
        Unary(UnaryExpr),
        Parenthesized(ParenthesizedExpr),
        TypeOperation(TypeOperationExpr),
    }
);

choice!(
    enum ChainSuffix {
        FnCall(AssociatedFnCallSuffix),
        StructField(AssociatedStructField),
    }
);

sequence!(
    struct AssociatedFnCallSuffix {
        dot: DotSymbol,
        ident: Ident,
        args_start: OpenParenthesisSymbol,
        #[force_error(true)]
        args: Repeated<FnArgGroup, 0, 1>,
        args_end: CloseParenthesisSymbol,
    }
);

impl NodeConfig for AssociatedFnCallSuffix {}

sequence!(
    #[allow(unused_mut)]
    struct AssociatedStructField {
        dot: DotSymbol,
        ident: Ident,
    }
);

impl NodeConfig for AssociatedStructField {}

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
    AlignofExpr, BoolLiteral, ParenthesizedExpr, SizeofExpr, VarIdentExpr,
};
use crate::language::expressions::unary::UnaryExpr;
use crate::language::items::{fn_, type_};
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

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
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
                let prefix_type_key = self.expr.type_(index)?.source()?.key()?;
                let field_name = &suffix.ident.slice;
                Some(format!("`{field_name}` field of {prefix_type_key}"))
            }
        }
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        match &**self.suffix.iter().next()? {
            ChainSuffix::FnCall(_) => {
                index.search(self, &self.source_key(index)?, sources::fn_criteria())
            }
            ChainSuffix::StructField(suffix) => {
                let type_ = self.expr.type_(index)?.source()?;
                let field = type_::field(type_, &suffix.ident.slice)?;
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

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
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
                let type_ = self
                    .expr
                    .type_(ctx.index)
                    .expect("internal error: chain prefix type not found")
                    .source()
                    .expect("internal error: invalid chain prefix");
                let prefix = self.expr.transpile(ctx);
                let suffix = type_::transpile_field_name(type_, &suffix.ident.slice);
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
        Bool(BoolLiteral),
        F32(F32Literal),
        U32(U32Literal),
        I32(I32Literal),
        FnCall(FnCallExpr),
        Constructor(ConstructorExpr),
        Var(VarIdentExpr),
        Unary(UnaryExpr),
        Parenthesized(ParenthesizedExpr),
        Alignof(AlignofExpr),
        Sizeof(SizeofExpr),
    }
);

impl NodeConfig for ChainPrefix {
    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        match self {
            Self::Bool(_)
            | Self::F32(_)
            | Self::U32(_)
            | Self::I32(_)
            | Self::Parenthesized(_)
            | Self::Constructor(_)
            | Self::Alignof(_)
            | Self::Sizeof(_) => Some(false),
            Self::Var(child) => child.is_ref(index),
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
            Self::Constructor(child) => child.type_(index),
            Self::Alignof(child) => child.type_(index),
            Self::Sizeof(child) => child.type_(index),
        }
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        match self {
            Self::Bool(child) => child.invalid_constant(index),
            Self::F32(child) => child.invalid_constant(index),
            Self::U32(child) => child.invalid_constant(index),
            Self::I32(child) => child.invalid_constant(index),
            Self::FnCall(child) => child.invalid_constant(index),
            Self::Constructor(child) => child.invalid_constant(index),
            Self::Var(child) => child.invalid_constant(index),
            Self::Unary(child) => child.invalid_constant(index),
            Self::Parenthesized(child) => child.invalid_constant(index),
            Self::Alignof(child) => child.invalid_constant(index),
            Self::Sizeof(child) => child.invalid_constant(index),
        }
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        match self {
            Self::Bool(child) => child.evaluate_constant(ctx),
            Self::F32(child) => child.evaluate_constant(ctx),
            Self::U32(child) => child.evaluate_constant(ctx),
            Self::I32(child) => child.evaluate_constant(ctx),
            Self::FnCall(child) => child.evaluate_constant(ctx),
            Self::Constructor(child) => child.evaluate_constant(ctx),
            Self::Var(child) => child.evaluate_constant(ctx),
            Self::Unary(child) => child.evaluate_constant(ctx),
            Self::Parenthesized(child) => child.evaluate_constant(ctx),
            Self::Alignof(child) => child.evaluate_constant(ctx),
            Self::Sizeof(child) => child.evaluate_constant(ctx),
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
            Self::Constructor(child) => child.transpile(ctx),
            Self::Alignof(child) => child.transpile(ctx),
            Self::Sizeof(child) => child.transpile(ctx),
        }
    }
}

choice!(
    enum ChainSuffix {
        FnCall(AssociatedFnCallSuffix),
        StructField(AssociatedStructField),
    }
);

impl NodeConfig for ChainSuffix {}

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

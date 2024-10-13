use crate::items::statement::StatementContext;
use crate::{
    errors, Asg, AsgBuffer, AsgFn, AsgFnParam, AsgFnSignature, AsgType, AsgVariable, Error, Result,
    ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN,
    NOT_FN, OR_FN, SUB_FN,
};
use shad_error::Span;
use shad_parser::{
    AstBinaryOperation, AstBinaryOperator, AstExpr, AstFnCall, AstIdent, AstLiteral,
    AstLiteralType, AstUnaryOperation, AstUnaryOperator,
};
use std::rc::Rc;

/// An analyzed expression definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgExpr {
    /// A literal.
    Literal(AsgLiteral),
    /// An identifier.
    Ident(AsgIdent),
    /// A call to a GPU function.
    FnCall(AsgFnCall),
}

impl AsgExpr {
    pub(crate) fn new(asg: &mut Asg, ctx: &StatementContext, expr: &AstExpr) -> Result<Self> {
        match expr {
            AstExpr::Literal(expr) => Ok(Self::Literal(AsgLiteral::new(asg, expr))),
            AstExpr::Ident(expr) => AsgIdent::new(asg, ctx, expr).map(Self::Ident),
            AstExpr::FnCall(expr) => AsgFnCall::new(asg, ctx, expr).map(Self::FnCall),
            AstExpr::UnaryOperation(expr) => {
                AsgFnCall::from_unary_op(asg, ctx, expr).map(Self::FnCall)
            }
            AstExpr::BinaryOperation(expr) => {
                AsgFnCall::from_binary_op(asg, ctx, expr).map(Self::FnCall)
            }
        }
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Literal(literal) => literal.ast.span,
            Self::Ident(ident) => ident.ast.span,
            Self::FnCall(call) => call.span,
        }
    }

    pub(crate) fn is_ref(&self) -> bool {
        match self {
            Self::Literal(_) => false,
            Self::Ident(_) => true,
            Self::FnCall(call) => call.fn_.is_returning_ref(),
        }
    }
}

/// An analyzed literal value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgLiteral {
    /// The parsed literal.
    pub ast: AstLiteral,
    /// The literal value.
    pub value: String,
    /// The literal type.
    pub type_: Rc<AsgType>,
}

impl AsgLiteral {
    fn new(asg: &Asg, literal: &AstLiteral) -> Self {
        Self {
            ast: literal.clone(),
            value: literal.value.replace('_', ""),
            type_: asg.types[Self::literal_type_str(literal)].clone(),
        }
    }

    fn literal_type_str(literal: &AstLiteral) -> &str {
        match literal.type_ {
            AstLiteralType::F32 => "f32",
            AstLiteralType::U32 => "u32",
            AstLiteralType::I32 => "i32",
            AstLiteralType::Bool => "bool",
        }
    }
}

/// An analyzed identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgIdent {
    /// The parsed identifier.
    pub ast: AstIdent,
    /// The identifier source.
    pub source: AsgIdentSource,
}

impl AsgIdent {
    pub(crate) fn new(asg: &mut Asg, ctx: &StatementContext, ident: &AstIdent) -> Result<Self> {
        let buffers_allowed = ctx.scope.are_buffers_allowed();
        Ok(Self {
            ast: ident.clone(),
            source: if let Some(variable) = ctx.variables.get(&ident.label) {
                AsgIdentSource::Var(variable.clone())
            } else if let Some(param) = ctx.param(&ident.label) {
                AsgIdentSource::Param(param.clone())
            } else if let (Some(buffer), true) = (asg.buffers.get(&ident.label), buffers_allowed) {
                AsgIdentSource::Buffer(buffer.clone())
            } else {
                asg.errors.push(errors::ident::not_found(asg, ident));
                return Err(Error);
            },
        })
    }
}

/// An analyzed identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgIdentSource {
    /// A buffer identifier.
    Buffer(Rc<AsgBuffer>),
    /// A variable identifier.
    Var(AsgVariable),
    /// A function parameter.
    Param(Rc<AsgFnParam>),
}

/// An analyzed function call.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnCall {
    /// The span of the call.
    pub span: Span,
    /// The function definition.
    pub fn_: Rc<AsgFn>,
    /// The function arguments.
    pub args: Vec<AsgExpr>,
    // TODO: still useful ?
}

impl AsgFnCall {
    pub(crate) fn new(asg: &mut Asg, ctx: &StatementContext, fn_call: &AstFnCall) -> Result<Self> {
        let args = fn_call
            .args
            .iter()
            .map(|arg| AsgExpr::new(asg, ctx, arg))
            .collect::<Result<Vec<AsgExpr>>>()?;
        let signature = AsgFnSignature::from_call(asg, &fn_call.name.label, &args)?;
        Ok(Self {
            span: fn_call.span,
            fn_: asg.find_function(fn_call.name.span, &signature)?.clone(),
            args,
        })
    }

    fn from_unary_op(
        asg: &mut Asg,
        ctx: &StatementContext,
        operation: &AstUnaryOperation,
    ) -> Result<Self> {
        let args = vec![AsgExpr::new(asg, ctx, &operation.expr)?];
        let fn_name = match operation.operator {
            AstUnaryOperator::Neg => NEG_FN,
            AstUnaryOperator::Not => NOT_FN,
        };
        let signature = AsgFnSignature::from_call(asg, fn_name, &args)?;
        Ok(Self {
            span: operation.span,
            fn_: asg
                .find_function(operation.operator_span, &signature)?
                .clone(),
            args,
        })
    }

    fn from_binary_op(
        asg: &mut Asg,
        ctx: &StatementContext,
        operation: &AstBinaryOperation,
    ) -> Result<Self> {
        let args = vec![
            AsgExpr::new(asg, ctx, &operation.left)?,
            AsgExpr::new(asg, ctx, &operation.right)?,
        ];
        let fn_name = match operation.operator {
            AstBinaryOperator::Add => ADD_FN,
            AstBinaryOperator::Sub => SUB_FN,
            AstBinaryOperator::Mul => MUL_FN,
            AstBinaryOperator::Div => DIV_FN,
            AstBinaryOperator::Mod => MOD_FN,
            AstBinaryOperator::Eq => EQ_FN,
            AstBinaryOperator::NotEq => NE_FN,
            AstBinaryOperator::GreaterThan => GT_FN,
            AstBinaryOperator::LessThan => LT_FN,
            AstBinaryOperator::GreaterThanOrEq => GE_FN,
            AstBinaryOperator::LessThanOrEq => LE_FN,
            AstBinaryOperator::And => AND_FN,
            AstBinaryOperator::Or => OR_FN,
        };
        let signature = AsgFnSignature::from_call(asg, fn_name, &args)?;
        Ok(Self {
            span: operation.span,
            fn_: asg
                .find_function(operation.operator_span, &signature)?
                .clone(),
            args,
        })
    }
}

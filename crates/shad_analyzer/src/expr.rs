use crate::statement::{AsgStatements, AsgVariable};
use crate::{
    errors, function, Asg, AsgBuffer, AsgFn, AsgFnParam, AsgFnSignature, AsgType, Error, Result,
    ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN,
    NOT_FN, OR_FN, SUB_FN,
};
use shad_error::{SemanticError, Span};
use shad_parser::{
    AstBinaryOperation, AstBinaryOperator, AstExpr, AstFnCall, AstFnQualifier, AstIdent,
    AstLiteral, AstLiteralType, AstUnaryOperation, AstUnaryOperator,
};
use std::rc::Rc;
use std::str::FromStr;

const F32_INT_PART_LIMIT: usize = 38;

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
    pub(crate) fn new(asg: &mut Asg, ctx: &AsgStatements<'_>, expr: &AstExpr) -> Result<Self> {
        match expr {
            AstExpr::Literal(expr) => Ok(Self::Literal(AsgLiteral::new(asg, expr))),
            AstExpr::Ident(expr) => AsgIdent::new(asg, ctx, expr).map(Self::Ident),
            AstExpr::FnCall(expr) => AsgFnCall::new(asg, ctx, expr)
                .map(|call| call.check_as_expr(asg))
                .map(Self::FnCall),
            AstExpr::UnaryOperation(expr) => AsgFnCall::from_unary_op(asg, ctx, expr)
                .map(|call| call.check_as_expr(asg))
                .map(Self::FnCall),
            AstExpr::BinaryOperation(expr) => AsgFnCall::from_binary_op(asg, ctx, expr)
                .map(|call| call.check_as_expr(asg))
                .map(Self::FnCall),
        }
    }
}

/// An analyzed literal value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgLiteral {
    /// The literal value.
    pub value: String,
    /// The literal type.
    pub type_: Rc<AsgType>,
}

impl AsgLiteral {
    fn new(asg: &mut Asg, literal: &AstLiteral) -> Self {
        let final_value = literal.value.replace('_', "");
        asg.errors
            .extend(Self::literal_error(asg, literal, &final_value));
        Self {
            value: final_value,
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

    // TODO: move in check phase
    fn literal_error(asg: &Asg, literal: &AstLiteral, final_value: &str) -> Option<SemanticError> {
        match literal.type_ {
            AstLiteralType::F32 => Self::f32_literal_error(asg, literal, final_value),
            AstLiteralType::U32 => {
                let digits = &final_value[..final_value.len() - 1];
                Self::int_literal_error::<u32>(asg, literal, digits, "u32")
            }
            AstLiteralType::I32 => Self::int_literal_error::<i32>(asg, literal, final_value, "i32"),
            AstLiteralType::Bool => None,
        }
    }

    // TODO: move in check phase
    // TODO: bug with span when "_" is used
    fn f32_literal_error(
        asg: &Asg,
        literal: &AstLiteral,
        final_value: &str,
    ) -> Option<SemanticError> {
        let digit_count = final_value
            .find('.')
            .expect("internal error: `.` not found in `f32` literal");
        (digit_count > F32_INT_PART_LIMIT).then(|| {
            let span = Span::new(literal.span.start, literal.span.start + digit_count);
            errors::literal::invalid_f32(asg, span, digit_count, F32_INT_PART_LIMIT)
        })
    }

    // TODO: move in check phase
    fn int_literal_error<T>(
        asg: &Asg,
        literal: &AstLiteral,
        final_value: &str,
        type_name: &str,
    ) -> Option<SemanticError>
    where
        T: FromStr,
    {
        let is_literal_invalid = T::from_str(final_value).is_err();
        is_literal_invalid.then(|| errors::literal::invalid_integer(asg, literal, type_name))
    }
}

/// An analyzed identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgIdent {
    /// A buffer identifier.
    Buffer(Rc<AsgBuffer>),
    /// A variable identifier.
    Var(Rc<AsgVariable>),
    /// A function parameter.
    Param(Rc<AsgFnParam>),
}

impl AsgIdent {
    pub(crate) fn new(asg: &mut Asg, ctx: &AsgStatements<'_>, ident: &AstIdent) -> Result<Self> {
        let are_buffers_allowed = ctx.scope.are_buffers_allowed();
        if let Some(variable) = ctx.variables.get(&ident.label) {
            Ok(Self::Var(variable.clone()))
        } else if let Some(param) = ctx.param(&ident.label) {
            Ok(Self::Param(param.clone()))
        } else if let (Some(buffer), true) = (asg.buffers.get(&ident.label), are_buffers_allowed) {
            Ok(Self::Buffer(buffer.clone()))
        } else {
            asg.errors.push(errors::ident::not_found(asg, ident));
            Err(Error)
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Buffer(buffer) => &buffer.ast.name.label,
            Self::Var(variable) => &variable.name.label,
            Self::Param(param) => &param.name.label,
        }
    }
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
}

impl AsgFnCall {
    pub(crate) fn new(asg: &mut Asg, ctx: &AsgStatements<'_>, fn_call: &AstFnCall) -> Result<Self> {
        let args = fn_call
            .args
            .iter()
            .map(|arg| AsgExpr::new(asg, ctx, arg))
            .collect::<Result<Vec<AsgExpr>>>()?;
        let signature = AsgFnSignature::from_call(asg, &fn_call.name.label, &args)?;
        Ok(Self {
            span: fn_call.span,
            fn_: function::find(asg, fn_call.name.span, &signature)?.clone(),
            args,
        }
        .check(asg, ctx))
    }

    fn from_unary_op(
        asg: &mut Asg,
        ctx: &AsgStatements<'_>,
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
            fn_: function::find(asg, operation.operator_span, &signature)?.clone(),
            args,
        }
        .check(asg, ctx))
    }

    fn from_binary_op(
        asg: &mut Asg,
        ctx: &AsgStatements<'_>,
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
            fn_: function::find(asg, operation.operator_span, &signature)?.clone(),
            args,
        }
        .check(asg, ctx))
    }

    // TODO: move in check phase
    fn check(self, asg: &mut Asg, ctx: &AsgStatements<'_>) -> Self {
        if self.fn_.ast.qualifier == AstFnQualifier::Buf && !ctx.scope.are_buffer_fns_allowed() {
            asg.errors
                .push(errors::fn_::invalid_buf_fn_call(asg, &self));
        }
        self
    }

    // TODO: move in check phase
    fn check_as_expr(self, asg: &mut Asg) -> Self {
        if self.fn_.return_type == Ok(None) {
            asg.errors
                .push(errors::fn_::call_without_return_type_in_expr(asg, &self));
        }
        self
    }

    // TODO: move in check phase
    pub(crate) fn check_as_statement(self, asg: &mut Asg) -> Self {
        if self.fn_.return_type != Ok(None) {
            asg.errors
                .push(errors::fn_::call_with_return_type_in_statement(asg, &self));
        }
        self
    }
}

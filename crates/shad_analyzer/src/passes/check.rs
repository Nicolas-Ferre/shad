use crate::errors::assignment;
use crate::items::function::{SPECIAL_BINARY_FNS, SPECIAL_UNARY_FNS};
use crate::result::result_ref;
use crate::{
    errors, Asg, AsgAssignment, AsgExpr, AsgFn, AsgFnBody, AsgFnCall, AsgLiteral, AsgReturn,
    AsgStatement, AsgVariableDefinition, TypeResolving,
};
use fxhash::FxHashMap;
use shad_error::{SemanticError, Span};
use shad_parser::{AstFnQualifier, AstLiteralType};
use std::rc::Rc;
use std::str::FromStr;

pub(crate) fn check(asg: &Asg) -> Vec<SemanticError> {
    let mut errors = vec![];
    for function in asg.functions.values() {
        let mut ctx = ErrorCheckContext::new(if function.ast.qualifier == AstFnQualifier::Buf {
            StatementScope::BufFnBody(function.clone())
        } else {
            StatementScope::FnBody(function.clone())
        });
        errors.extend(function.check(asg, &mut ctx));
        errors.extend(asg.function_bodies[function.index].check(asg, &mut ctx));
    }
    for buffer in asg.buffers.values() {
        let mut ctx = ErrorCheckContext::new(StatementScope::BufferExpr);
        if let Ok(expr) = &buffer.expr {
            errors.extend(expr.check(asg, &mut ctx));
        }
    }
    for statements in &asg.run_blocks {
        for statement in statements {
            let mut ctx = ErrorCheckContext::new(StatementScope::RunBlock);
            errors.extend(statement.check(asg, &mut ctx));
        }
    }
    errors
}

trait ErrorCheck {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError>;
}

#[derive(Debug)]
struct ErrorCheckContext {
    scope: StatementScope,
    return_span: Option<Span>,
    is_statement_after_return_found: bool,
    expr_level: usize,
}

impl ErrorCheckContext {
    fn new(scope: StatementScope) -> Self {
        Self {
            scope,
            return_span: None,
            is_statement_after_return_found: false,
            expr_level: 0,
        }
    }

    fn is_in_expr(&self) -> bool {
        self.expr_level > 0
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum StatementScope {
    BufferExpr,
    RunBlock,
    BufFnBody(Rc<AsgFn>),
    FnBody(Rc<AsgFn>),
}

impl StatementScope {
    pub(crate) fn fn_(&self) -> Option<&Rc<AsgFn>> {
        match self {
            Self::BufferExpr | Self::RunBlock => None,
            Self::FnBody(fn_) | Self::BufFnBody(fn_) => Some(fn_),
        }
    }

    pub(crate) fn are_buffers_allowed(&self) -> bool {
        match self {
            Self::BufferExpr | Self::RunBlock | Self::BufFnBody(_) => true,
            Self::FnBody(_) => false,
        }
    }

    fn are_buffer_fns_allowed(&self) -> bool {
        match self {
            Self::RunBlock | Self::BufFnBody(_) => true,
            Self::BufferExpr | Self::FnBody(_) => false,
        }
    }
}

impl ErrorCheck for AsgFn {
    fn check(&self, asg: &Asg, _ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors = check_duplicated_params(asg, self);
        if SPECIAL_UNARY_FNS.contains(&self.ast.name.label.as_str()) {
            errors.extend(check_param_count(asg, self, 1));
        }
        if SPECIAL_BINARY_FNS.contains(&self.ast.name.label.as_str()) {
            errors.extend(check_param_count(asg, self, 2));
        }
        errors
    }
}

impl ErrorCheck for AsgFnBody {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors: Vec<_> = self
            .statements
            .iter()
            .flat_map(|statement| statement.check(asg, ctx))
            .collect();
        if self.fn_.ast.qualifier != AstFnQualifier::Gpu
            && ctx.return_span.is_none()
            && matches!(&self.fn_.return_type, Ok(Some(_)))
        {
            errors.push(errors::return_::missing_return(asg, &self.fn_));
        }
        errors
    }
}

impl ErrorCheck for AsgStatement {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors: Vec<_> = check_statement_after_return(asg, ctx, self)
            .into_iter()
            .collect();
        errors.extend(match self {
            Self::Var(var) => var.check(asg, ctx),
            Self::Assignment(assignment) => assignment.check(asg, ctx),
            Self::Return(return_) => return_.check(asg, ctx),
            Self::FnCall(Ok(call)) => call.check(asg, ctx),
            Self::FnCall(Err(_)) => vec![], // no-coverage (failed analysis)
        });
        errors
    }
}

impl ErrorCheck for AsgVariableDefinition {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        self.expr
            .as_ref()
            .map(|expr| expr.check(asg, ctx))
            .unwrap_or_default()
    }
}

impl ErrorCheck for AsgAssignment {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors = self
            .expr
            .as_ref()
            .map(|expr| expr.check(asg, ctx))
            .unwrap_or_default();
        if let (Ok(assigned), Ok(assigned_type), Ok(expr_type)) = (
            &self.assigned,
            result_ref(&self.assigned).and_then(|value| value.type_(asg)),
            result_ref(&self.expr).and_then(|expr| expr.type_(asg)),
        ) {
            if assigned_type != expr_type {
                errors.push(assignment::invalid_type(
                    asg,
                    self,
                    assigned,
                    assigned_type,
                    expr_type,
                ));
            }
        }
        errors
    }
}

impl ErrorCheck for AsgReturn {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors = self
            .expr
            .as_ref()
            .map(|expr| expr.check(asg, ctx))
            .unwrap_or_default();
        if let Some(fn_) = ctx.scope.fn_() {
            ctx.return_span = Some(self.ast.span);
            errors.extend(check_return_type(asg, fn_, self));
        } else {
            errors.push(errors::return_::outside_fn(asg, self));
        }
        errors
    }
}

impl ErrorCheck for AsgExpr {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        ctx.expr_level += 1;
        let errors = match self {
            Self::Literal(literal) => literal.check(asg, ctx),
            Self::FnCall(call) => call.check(asg, ctx),
            Self::Ident(_) => vec![],
        };
        ctx.expr_level -= 1;
        errors
    }
}

impl ErrorCheck for AsgLiteral {
    fn check(&self, asg: &Asg, _ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let error = match self.ast.type_ {
            AstLiteralType::F32 => check_f32_literal(asg, self),
            AstLiteralType::U32 => check_int_literal::<u32>(asg, self, "u32"),
            AstLiteralType::I32 => check_int_literal::<i32>(asg, self, "i32"),
            AstLiteralType::Bool => None,
        };
        error.into_iter().collect()
    }
}

impl ErrorCheck for AsgFnCall {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        let mut errors: Vec<_> = self
            .args
            .iter()
            .flat_map(|arg| arg.check(asg, ctx))
            .collect();
        if self.fn_.ast.qualifier == AstFnQualifier::Buf && !ctx.scope.are_buffer_fns_allowed() {
            errors.push(errors::fn_::invalid_buf_fn_call(asg, self));
        }
        if ctx.is_in_expr() && self.fn_.return_type == Ok(None) {
            errors.push(errors::fn_::call_without_return_type_in_expr(asg, self));
        }
        if !ctx.is_in_expr() && self.fn_.return_type != Ok(None) {
            errors.push(errors::fn_::call_with_return_type_in_statement(asg, self));
        }
        for (arg, param) in self.args.iter().zip(&self.fn_.ast.params) {
            if let (Some(ref_span), false) = (param.ref_span, matches!(arg, AsgExpr::Ident(_))) {
                errors.push(errors::fn_::invalid_ref(asg, arg, ref_span));
            }
        }
        errors
    }
}

fn check_duplicated_params(asg: &Asg, fn_: &AsgFn) -> Vec<SemanticError> {
    let mut names = FxHashMap::default();
    fn_.params
        .iter()
        .filter_map(|param| {
            names
                .insert(&param.ast.name.label, &param.ast.name)
                .map(|existing_param| (&param.ast.name, existing_param))
        })
        .map(|(param1, param2)| errors::fn_::duplicated_param(asg, param1, param2))
        .collect()
}

fn check_param_count(asg: &Asg, fn_: &AsgFn, expected_count: usize) -> Option<SemanticError> {
    (fn_.params.len() != expected_count)
        .then(|| errors::fn_::invalid_param_count(asg, fn_, expected_count))
}

fn check_statement_after_return(
    asg: &Asg,
    ctx: &mut ErrorCheckContext,
    statement: &AsgStatement,
) -> Option<SemanticError> {
    if let (Some(return_span), Ok(statement_span), false) = (
        ctx.return_span,
        statement.span(),
        ctx.is_statement_after_return_found,
    ) {
        ctx.is_statement_after_return_found = true;
        Some(errors::return_::statement_after(
            asg,
            statement_span,
            return_span,
        ))
    } else {
        None
    }
}

fn check_return_type(asg: &Asg, fn_: &AsgFn, return_: &AsgReturn) -> Option<SemanticError> {
    if let (Ok(actual_type), Ok(expected_type)) = (
        result_ref(&return_.expr).and_then(|expr| expr.type_(asg).cloned()),
        fn_.return_type.as_ref(),
    ) {
        if let Some(expected_type) = expected_type {
            (&actual_type != expected_type).then(|| {
                errors::return_::invalid_type(asg, return_, fn_, &actual_type, expected_type)
            })
        } else {
            Some(errors::return_::no_return_type(asg, return_))
        }
    } else {
        None
    }
}

fn check_f32_literal(asg: &Asg, literal: &AsgLiteral) -> Option<SemanticError> {
    const F32_INT_PART_LIMIT: usize = 38;
    let digit_count = int_part_digit_count(&literal.value);
    (digit_count > F32_INT_PART_LIMIT).then(|| {
        let offset = int_part_digit_count(&literal.ast.value);
        let span = Span::new(literal.ast.span.start, literal.ast.span.start + offset);
        errors::literal::invalid_f32(asg, span, digit_count, F32_INT_PART_LIMIT)
    })
}

fn check_int_literal<T>(asg: &Asg, literal: &AsgLiteral, type_name: &str) -> Option<SemanticError>
where
    T: FromStr,
{
    let value = if type_name == "u32" {
        &literal.value[..literal.value.len() - 1]
    } else {
        &literal.value
    };
    T::from_str(value)
        .is_err()
        .then(|| errors::literal::invalid_integer(asg, literal, type_name))
}

fn int_part_digit_count(float: &str) -> usize {
    float
        .find('.')
        .expect("internal error: `.` not found in `f32` literal")
}

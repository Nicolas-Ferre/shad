use crate::errors::assignment;
use crate::passes::type_resolving::TypeResolving;
use crate::result::result_ref;
use crate::{
    errors, Asg, AsgBuffer, AsgExpr, AsgFn, AsgFnCall, AsgFnParam, AsgIdent, Error, Result,
};
use fxhash::FxHashMap;
use shad_error::Span;
use shad_parser::{AstAssignment, AstIdent, AstReturn, AstStatement, AstVarDefinition};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct AsgStatements<'a> {
    next_variable_index: usize,
    pub(crate) statements: Vec<AsgStatement>,
    pub(crate) variables: FxHashMap<String, Rc<AsgVariable>>,
    pub(crate) scope: AsgStatementScopeType<'a>,
    pub(crate) return_span: Option<Span>,
    pub(crate) is_statement_after_return_found: bool,
}

impl<'a> AsgStatements<'a> {
    pub(crate) fn buffer_scope() -> Self {
        Self {
            next_variable_index: 0,
            statements: vec![],
            variables: FxHashMap::default(),
            scope: AsgStatementScopeType::BufferExpr,
            return_span: None,
            is_statement_after_return_found: false,
        }
    }

    pub(crate) fn analyze(
        asg: &mut Asg,
        statements: &[AstStatement],
        scope: AsgStatementScopeType<'a>,
    ) -> Vec<AsgStatement> {
        let mut parsed = Self {
            next_variable_index: 0,
            statements: vec![],
            variables: FxHashMap::default(),
            scope,
            return_span: None,
            is_statement_after_return_found: false,
        };
        for ast_statement in statements {
            let statement = AsgStatement::new(asg, &mut parsed, ast_statement);
            parsed.statements.extend(statement);
        }
        parsed.statements
    }

    pub(crate) fn param(&self, name: &str) -> Option<&Rc<AsgFnParam>> {
        self.scope
            .fn_()
            .map(|fn_| fn_.params.as_slice())
            .unwrap_or_default()
            .iter()
            .find(|param| param.name.label == name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum AsgStatementScopeType<'a> {
    BufferExpr,
    RunBlock,
    BufFnBody(&'a AsgFn),
    FnBody(&'a AsgFn),
}

impl<'a> AsgStatementScopeType<'a> {
    pub(crate) fn are_buffers_allowed(self) -> bool {
        match self {
            Self::BufferExpr | Self::RunBlock | Self::BufFnBody(_) => true,
            Self::FnBody(_) => false,
        }
    }

    pub(crate) fn are_buffer_fns_allowed(self) -> bool {
        match self {
            Self::RunBlock | Self::BufFnBody(_) => true,
            Self::BufferExpr | Self::FnBody(_) => false,
        }
    }

    pub(crate) fn fn_(self) -> Option<&'a AsgFn> {
        match self {
            Self::BufferExpr | Self::RunBlock => None,
            Self::FnBody(fn_) | Self::BufFnBody(fn_) => Some(fn_),
        }
    }
}

/// An analyzed statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgStatement {
    /// A variable definition.
    Var(Rc<AsgVariable>),
    /// A variable assignment.
    Assignment(AsgAssignment),
    /// A return statement.
    Return(Result<AsgExpr>),
    /// An expression statement.
    FnCall(Result<AsgFnCall>),
}

impl AsgStatement {
    fn new(asg: &mut Asg, ctx: &mut AsgStatements<'_>, statement: &AstStatement) -> Result<Self> {
        if let (Some(return_span), false) = (ctx.return_span, ctx.is_statement_after_return_found) {
            // TODO: move in check phase
            ctx.is_statement_after_return_found = true;
            asg.errors.push(errors::return_::statement_after(
                asg,
                statement,
                return_span,
            ));
            Err(Error)
        } else {
            Ok(match statement {
                AstStatement::Var(statement) => Self::Var(AsgVariable::new(asg, ctx, statement)),
                AstStatement::Assignment(statement) => {
                    Self::Assignment(AsgAssignment::new(asg, ctx, statement))
                }
                AstStatement::Return(statement) => {
                    Self::Return(Self::analyze_return(asg, ctx, statement))
                }
                AstStatement::FnCall(statement) => Self::FnCall(
                    AsgFnCall::new(asg, ctx, &statement.call)
                        .map(|call| call.check_as_statement(asg)),
                ),
            })
        }
    }

    fn analyze_return(
        asg: &mut Asg,
        ctx: &mut AsgStatements<'_>,
        statement: &AstReturn,
    ) -> Result<AsgExpr> {
        // TODO: move in check phase
        if ctx.scope.fn_().is_none() {
            asg.errors.push(errors::return_::outside_fn(asg, statement));
        } else {
            ctx.return_span = Some(statement.span);
        }
        let expr = AsgExpr::new(asg, ctx, &statement.expr)?;
        // TODO: move in check phase
        if let (Ok(actual_type), Some(expected_type), Some(fn_)) = (
            expr.type_(asg),
            ctx.scope.fn_().and_then(|f| f.return_type.as_ref().ok()),
            ctx.scope.fn_(),
        ) {
            if let Some(expected_type) = expected_type {
                if actual_type != expected_type {
                    asg.errors.push(errors::return_::invalid_type(
                        asg,
                        statement,
                        fn_,
                        actual_type,
                        expected_type,
                    ));
                }
            } else {
                asg.errors
                    .push(errors::return_::no_return_type(asg, statement));
            }
        }
        Ok(expr)
    }
}

/// An analyzed assignment statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgAssignment {
    /// The updated variable.
    pub assigned: Result<AsgIdent>,
    /// The new value.
    pub expr: Result<AsgExpr>,
    /// The span of the updated variable.
    pub assigned_span: Span,
    /// The span of the new value.
    pub expr_span: Span,
}

impl AsgAssignment {
    fn new(asg: &mut Asg, ctx: &AsgStatements<'_>, assignment: &AstAssignment) -> Self {
        let expr = AsgExpr::new(asg, ctx, &assignment.expr);
        Self::checked(
            Self {
                assigned: AsgIdent::new(asg, ctx, &assignment.value),
                expr,
                assigned_span: assignment.value.span,
                expr_span: assignment.expr.span(),
            },
            asg,
        )
    }

    pub(crate) fn buffer_init(asg: &mut Asg, buffer: &Rc<AsgBuffer>) -> Self {
        Self::checked(
            Self {
                assigned: Ok(AsgIdent::Buffer(buffer.clone())),
                expr: buffer.expr.clone(),
                assigned_span: buffer.ast.name.span,
                expr_span: buffer.ast.value.span(),
            },
            asg,
        )
    }

    // TODO: move in check phase
    fn checked(self, asg: &mut Asg) -> Self {
        if let (Ok(assigned), Ok(assigned_type), Ok(expr_type)) = (
            &self.assigned,
            result_ref(&self.assigned).and_then(|value| value.type_(asg)),
            result_ref(&self.expr).and_then(|expr| expr.type_(asg)),
        ) {
            if assigned_type != expr_type {
                asg.errors.push(assignment::invalid_type(
                    asg,
                    &self,
                    assigned,
                    assigned_type,
                    expr_type,
                ));
            }
        }
        self
    }
}

/// An analyzed variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgVariable {
    /// The name of the variable.
    pub name: AstIdent,
    /// The unique index of the variable in the shader.
    pub index: usize,
    /// The initial value of the variable.
    pub expr: Result<AsgExpr>,
}

impl AsgVariable {
    fn new(asg: &mut Asg, ctx: &mut AsgStatements<'_>, variable: &AstVarDefinition) -> Rc<Self> {
        let final_variable = Rc::new(Self {
            name: variable.name.clone(),
            index: ctx.next_variable_index,
            expr: AsgExpr::new(asg, ctx, &variable.expr),
        });
        ctx.next_variable_index += 1;
        ctx.variables
            .insert(variable.name.label.clone(), final_variable.clone());
        final_variable
    }
}

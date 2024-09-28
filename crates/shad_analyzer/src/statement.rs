use crate::{utils, Asg, AsgBuffer, AsgExpr, AsgFn, AsgFnParam, AsgIdent, AsgType};
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
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

    pub(crate) fn parse(
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
    Return(Result<AsgExpr, ()>),
}

impl AsgStatement {
    fn new(
        asg: &mut Asg,
        ctx: &mut AsgStatements<'_>,
        statement: &AstStatement,
    ) -> Result<Self, ()> {
        if let (Some(return_span), false) = (ctx.return_span, ctx.is_statement_after_return_found) {
            ctx.is_statement_after_return_found = true;
            asg.errors.push(Self::statement_after_return_error(
                asg,
                statement,
                return_span,
            ));
            Err(())
        } else {
            Ok(match statement {
                AstStatement::Var(statement) => Self::Var(AsgVariable::new(asg, ctx, statement)),
                AstStatement::Assignment(statement) => {
                    Self::Assignment(AsgAssignment::new(asg, ctx, statement))
                }
                AstStatement::Return(statement) => {
                    Self::Return(Self::analyze_return(asg, ctx, statement))
                }
            })
        }
    }

    pub(crate) fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Assignment(statement) => statement.buffers(asg),
            Self::Var(statement) => statement.buffers(asg),
            Self::Return(statement) => statement
                .as_ref()
                .map(|expr| expr.buffers(asg))
                .unwrap_or_default(),
        }
    }

    pub(crate) fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        match self {
            Self::Assignment(statement) => statement.functions(asg),
            Self::Var(statement) => statement.functions(asg),
            Self::Return(statement) => statement
                .as_ref()
                .map(|expr| expr.functions(asg))
                .unwrap_or_default(),
        }
    }

    fn analyze_return(
        asg: &mut Asg,
        ctx: &mut AsgStatements<'_>,
        statement: &AstReturn,
    ) -> Result<AsgExpr, ()> {
        if ctx.scope.fn_().is_none() {
            asg.errors
                .push(Self::return_outside_fn_error(asg, statement));
        } else {
            ctx.return_span = Some(statement.span);
        }
        let expr = AsgExpr::new(asg, ctx, &statement.expr)?;
        if let (Ok(actual_type), Some(expected_type), Some(fn_)) = (
            expr.type_(asg),
            ctx.scope.fn_().and_then(|f| f.return_type.as_ref().ok()),
            ctx.scope.fn_(),
        ) {
            if actual_type != expected_type {
                asg.errors.push(Self::mismatch_type(
                    asg,
                    statement,
                    fn_,
                    actual_type,
                    expected_type,
                ));
            }
        }
        Ok(expr)
    }

    fn return_outside_fn_error(asg: &Asg, statement: &AstReturn) -> SemanticError {
        SemanticError::new(
            "`return` statement used outside function",
            vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: statement.span,
                text: "invalid statement".into(),
            }],
            &asg.code,
            &asg.path,
        )
    }

    fn mismatch_type(
        asg: &Asg,
        statement: &AstReturn,
        fn_: &AsgFn,
        actual: &AsgType,
        expected: &AsgType,
    ) -> SemanticError {
        SemanticError::new(
            "invalid type for returned expression",
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: statement.expr.span(),
                    text: format!("expression of type `{}`", actual.name.as_str()),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: fn_.ast.return_type.span,
                    text: format!("expected type `{}`", expected.name.as_str()),
                },
            ],
            &asg.code,
            &asg.path,
        )
    }

    fn statement_after_return_error(
        asg: &Asg,
        statement: &AstStatement,
        return_span: Span,
    ) -> SemanticError {
        SemanticError::new(
            "statement found after `return` statement",
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: statement.span(),
                    text: "this statement cannot be defined after a `return` statement".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: return_span,
                    text: "`return` statement defined here".into(),
                },
            ],
            &asg.code,
            &asg.path,
        )
    }
}

/// An analyzed assignment statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgAssignment {
    /// The updated variable.
    pub assigned: Result<AsgIdent, ()>,
    /// The new value.
    pub expr: Result<AsgExpr, ()>,
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

    fn checked(self, asg: &mut Asg) -> Self {
        if let (Ok(assigned), Ok(assigned_type), Ok(expr_type)) = (
            &self.assigned,
            utils::result_ref(&self.assigned).and_then(|e| e.type_(asg)),
            utils::result_ref(&self.expr).and_then(|e| e.type_(asg)),
        ) {
            if assigned_type != expr_type {
                asg.errors.push(self.mismatching_type_error(
                    asg,
                    assigned,
                    assigned_type,
                    expr_type,
                ));
            }
        }
        self
    }

    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        let mut buffers = utils::result_ref(&self.assigned)
            .map(AsgIdent::buffers)
            .unwrap_or_default();
        buffers.append(
            &mut utils::result_ref(&self.expr)
                .map(|expr| expr.buffers(asg))
                .unwrap_or_default(),
        );
        buffers
    }

    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        utils::result_ref(&self.expr)
            .map(|expr| expr.functions(asg))
            .unwrap_or_default()
    }

    pub(crate) fn mismatching_type_error(
        &self,
        asg: &Asg,
        assigned: &AsgIdent,
        assigned_type: &AsgType,
        expr_type: &AsgType,
    ) -> SemanticError {
        SemanticError::new(
            format!(
                "expression assigned to `{}` has invalid type",
                assigned.name()
            ),
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: self.expr_span,
                    text: format!("expression of type `{}`", expr_type.name.as_str()),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: self.assigned_span,
                    text: format!("expected type `{}`", assigned_type.name.as_str()),
                },
            ],
            &asg.code,
            &asg.path,
        )
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
    pub expr: Result<AsgExpr, ()>,
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

    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        utils::result_ref(&self.expr)
            .map(|expr| expr.buffers(asg))
            .unwrap_or_default()
    }

    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        utils::result_ref(&self.expr)
            .map(|expr| expr.functions(asg))
            .unwrap_or_default()
    }
}

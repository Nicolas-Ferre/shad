use crate::passes::check::StatementScope;
use crate::result::result_ref;
use crate::{Asg, AsgBuffer, AsgExpr, AsgFnCall, AsgFnParam, AsgIdent, AsgIdentSource, Result};
use fxhash::FxHashMap;
use shad_error::Span;
use shad_parser::{AstAssignment, AstReturn, AstStatement, AstVarDefinition};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct StatementContext {
    pub(crate) scope: StatementScope,
    pub(crate) variables: FxHashMap<String, Rc<AsgVariable>>,
}

impl StatementContext {
    pub(crate) fn from_scope(scope: StatementScope) -> Self {
        Self {
            scope,
            variables: FxHashMap::default(),
        }
    }

    pub(crate) fn analyze(
        asg: &mut Asg,
        statements: &[AstStatement],
        scope: StatementScope,
    ) -> Vec<AsgStatement> {
        let mut parsed = Self {
            variables: FxHashMap::default(),
            scope,
        };
        statements
            .iter()
            .map(|statement| AsgStatement::new(asg, &mut parsed, statement))
            .collect()
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

/// An analyzed statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgStatement {
    /// A variable definition.
    Var(AsgVariableDefinition),
    /// A variable assignment.
    Assignment(AsgAssignment),
    /// A return statement.
    Return(AsgReturn),
    /// An expression statement.
    FnCall(Result<AsgFnCall>),
}

impl AsgStatement {
    fn new(asg: &mut Asg, ctx: &mut StatementContext, statement: &AstStatement) -> Self {
        match statement {
            AstStatement::Var(var) => Self::Var(AsgVariableDefinition::new(asg, ctx, var)),
            AstStatement::Assignment(assignment) => {
                Self::Assignment(AsgAssignment::new(asg, ctx, assignment))
            }
            AstStatement::Return(return_) => Self::Return(AsgReturn::new(asg, ctx, return_)),
            AstStatement::FnCall(call) => Self::FnCall(AsgFnCall::new(asg, ctx, &call.call)),
        }
    }

    pub(crate) fn span(&self) -> Result<Span> {
        match self {
            Self::Var(statement) => Ok(statement.var.ast.span),
            Self::Assignment(statement) => Ok(statement
                .ast
                .as_ref()
                .expect("internal error: buffer init has no parsed statement")
                .span),
            Self::Return(statement) => Ok(statement.ast.span),
            Self::FnCall(statement) => result_ref(statement).map(|s| s.span),
        }
    }
}

/// An analyzed assignment statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgAssignment {
    /// The parsed assignment.
    pub ast: Option<AstAssignment>,
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
    fn new(asg: &mut Asg, ctx: &StatementContext, assignment: &AstAssignment) -> Self {
        Self {
            ast: Some(assignment.clone()),
            assigned: AsgIdent::new(asg, ctx, &assignment.value),
            expr: AsgExpr::new(asg, ctx, &assignment.expr),
            assigned_span: assignment.value.span,
            expr_span: assignment.expr.span(),
        }
    }

    pub(crate) fn buffer_init(buffer: &Rc<AsgBuffer>) -> Self {
        Self {
            ast: None,
            assigned: Ok(AsgIdent {
                ast: buffer.ast.name.clone(),
                source: AsgIdentSource::Buffer(buffer.clone()),
            }),
            expr: buffer.expr.clone(),
            assigned_span: buffer.ast.name.span,
            expr_span: buffer.ast.value.span(),
        }
    }
}

/// An analyzed variable definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgVariableDefinition {
    /// The variable details.
    pub var: Rc<AsgVariable>,
    /// The value assigned to the variable.
    pub expr: Result<AsgExpr>,
}

impl AsgVariableDefinition {
    pub(crate) fn new(
        asg: &mut Asg,
        ctx: &mut StatementContext,
        variable: &AstVarDefinition,
    ) -> Self {
        let var = AsgVariable::new(asg, ctx, variable);
        Self {
            expr: var.expr.clone(),
            var,
        }
    }
}

/// An analyzed variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgVariable {
    /// The parsed variable definition.
    pub ast: AstVarDefinition,
    /// The unique index of the variable in the shader.
    pub index: usize,
    /// The initial value of the variable.
    pub expr: Result<AsgExpr>,
}

impl AsgVariable {
    fn new(asg: &mut Asg, ctx: &mut StatementContext, variable: &AstVarDefinition) -> Rc<Self> {
        let final_variable = Rc::new(Self {
            ast: variable.clone(),
            index: asg.next_var_index(),
            expr: AsgExpr::new(asg, ctx, &variable.expr),
        });
        ctx.variables
            .insert(variable.name.label.clone(), final_variable.clone());
        final_variable
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// An analyzed return statement.
pub struct AsgReturn {
    /// The span of the return statement.
    pub ast: AstReturn,
    /// The span of the return statement.
    pub expr: Result<AsgExpr>,
}

impl AsgReturn {
    fn new(asg: &mut Asg, ctx: &StatementContext, return_: &AstReturn) -> Self {
        Self {
            ast: return_.clone(),
            expr: AsgExpr::new(asg, ctx, &return_.expr),
        }
    }
}

use crate::passes::check::StatementScope;
use crate::result::result_ref;
use crate::{
    Asg, AsgBuffer, AsgExpr, AsgFnCall, AsgFnParam, AsgIdent, AsgIdentSource, Error, Result,
};
use fxhash::FxHashMap;
use shad_error::Span;
use shad_parser::{
    AstAssignment, AstIdent, AstLeftValue, AstReturn, AstStatement, AstVarDefinition,
};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct StatementContext {
    pub(crate) scope: StatementScope,
    pub(crate) variables: FxHashMap<String, AsgVariable>,
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
            .find(|param| param.ast.name.label == name)
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
            Self::Var(statement) => Ok(statement.var.span),
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
    /// The updated value.
    pub assigned: Result<AsgLeftValue>,
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
            assigned: AsgLeftValue::new(asg, ctx, &assignment.value),
            expr: AsgExpr::new(asg, ctx, &assignment.expr),
            assigned_span: assignment.value.span(),
            expr_span: assignment.expr.span(),
        }
    }

    pub(crate) fn buffer_init(buffer: &Rc<AsgBuffer>) -> Self {
        Self {
            ast: None,
            assigned: Ok(AsgLeftValue::Ident(AsgIdent {
                ast: buffer.ast.name.clone(),
                source: AsgIdentSource::Buffer(buffer.clone()),
            })),
            expr: buffer.expr.clone(),
            assigned_span: buffer.ast.name.span,
            expr_span: buffer.ast.value.span(),
        }
    }
}

/// An analyzed left value in an assignment.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgLeftValue {
    /// An identifier.
    Ident(AsgIdent),
    /// A function call.
    FnCall(AsgFnCall),
}

impl AsgLeftValue {
    fn new(asg: &mut Asg, ctx: &StatementContext, value: &AstLeftValue) -> Result<Self> {
        match value {
            AstLeftValue::Ident(ident) => AsgIdent::new(asg, ctx, ident).map(Self::Ident),
            AstLeftValue::FnCall(call) => AsgFnCall::new(asg, ctx, call).map(Self::FnCall),
        }
    }
}

impl TryFrom<AsgExpr> for AsgLeftValue {
    type Error = Error;

    fn try_from(value: AsgExpr) -> std::result::Result<Self, Self::Error> {
        match value {
            AsgExpr::Literal(_) | AsgExpr::FnCall(_) => Err(Error),
            AsgExpr::Ident(ident) => Ok(Self::Ident(ident)),
        }
    }
}

/// An analyzed variable definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgVariableDefinition {
    /// The variable details.
    pub var: AsgVariable,
    /// The value assigned to the variable.
    pub expr: Result<Box<AsgExpr>>,
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

    pub(crate) fn inlined(asg: &mut Asg, expr: &AsgExpr) -> Self {
        Self {
            var: AsgVariable {
                span: expr.span(),
                name: AstIdent {
                    span: expr.span(),
                    label: "inlined".to_string(),
                },
                index: asg.next_var_index(),
                inline_index: None,
                expr: Ok(Box::new(expr.clone())),
            },
            expr: Ok(Box::new(expr.clone())),
        }
    }
}

/// An analyzed variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgVariable {
    /// The span of the variable definition.
    pub span: Span,
    /// The name of the variable.
    pub name: AstIdent,
    /// The unique index of the variable in the shader.
    pub index: usize,
    /// A secondary unique index in case of inlining.
    pub inline_index: Option<usize>,
    /// The initial value of the variable.
    pub expr: Result<Box<AsgExpr>>,
}

impl AsgVariable {
    fn new(asg: &mut Asg, ctx: &mut StatementContext, variable: &AstVarDefinition) -> Self {
        let final_variable = Self {
            span: variable.span,
            name: variable.name.clone(),
            index: asg.next_var_index(),
            inline_index: None,
            expr: AsgExpr::new(asg, ctx, &variable.expr).map(Box::new),
        };
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

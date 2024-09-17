use crate::{utils, Asg, AsgBuffer, AsgExpr, AsgIdent, AsgType};
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstAssignment, AstExpr, AstIdent, AstRunItem, AstStatement, AstVarDefinition};
use std::rc::Rc;

/// An analyzed compute shader.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgComputeShader {
    /// The buffers used in the shader.
    pub buffers: FxHashMap<String, Rc<AsgBuffer>>,
    /// The statements of the shader main function.
    pub statements: Vec<AsgStatement>,
    /// The name of the shader.
    pub name: String,
}

impl AsgComputeShader {
    pub(crate) fn buffer_init(asg: &mut Asg, buffer: &Rc<AsgBuffer>, expr: &AstExpr) -> Self {
        let statements = vec![AsgStatement::Assignment(AsgAssignment::buffer_init(
            asg, buffer, expr,
        ))];
        Self {
            buffers: Self::buffers(&statements),
            statements,
            name: format!("buffer_init:{}", buffer.name.label),
        }
    }

    pub(crate) fn step(asg: &mut Asg, ast_run: &AstRunItem) -> Self {
        let mut ctx = AsgStatements::default();
        for ast_statement in &ast_run.statements {
            let statement = AsgStatement::new(asg, &mut ctx, ast_statement);
            ctx.statements.push(statement);
        }
        Self {
            buffers: Self::buffers(&ctx.statements),
            statements: ctx.statements,
            name: "run".into(),
        }
    }

    /// Returns all buffers used in the shader.
    fn buffers(statements: &[AsgStatement]) -> FxHashMap<String, Rc<AsgBuffer>> {
        statements.iter().flat_map(AsgStatement::buffers).collect()
    }
}

/// Analyzed statements.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct AsgStatements {
    statements: Vec<AsgStatement>,
    pub(crate) variables: FxHashMap<String, Rc<AsgVariable>>,
    next_variable_index: usize,
}

/// An analyzed statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgStatement {
    /// A variable definition.
    Var(Rc<AsgVariable>),
    /// A variable assignment.
    Assignment(AsgAssignment),
}

impl AsgStatement {
    fn new(asg: &mut Asg, ctx: &mut AsgStatements, statement: &AstStatement) -> Self {
        match statement {
            AstStatement::Var(statement) => Self::Var(AsgVariable::new(asg, ctx, statement)),
            AstStatement::Assignment(statement) => {
                Self::Assignment(AsgAssignment::new(asg, ctx, statement))
            }
        }
    }

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        match self {
            Self::Assignment(statement) => statement.buffers(),
            Self::Var(statement) => statement.buffers(),
        }
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
    fn new(asg: &mut Asg, ctx: &AsgStatements, assignment: &AstAssignment) -> Self {
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

    fn buffer_init(asg: &mut Asg, buffer: &Rc<AsgBuffer>, expr: &AstExpr) -> Self {
        Self::checked(
            Self {
                assigned: Ok(AsgIdent::Buffer(buffer.clone())),
                expr: buffer.expr.clone(),
                assigned_span: buffer.name.span,
                expr_span: expr.span(),
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

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        let mut buffers = utils::result_ref(&self.assigned)
            .map(AsgIdent::buffers)
            .unwrap_or_default();
        buffers.append(
            &mut utils::result_ref(&self.expr)
                .map(AsgExpr::buffers)
                .unwrap_or_default(),
        );
        buffers
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
                    text: format!("expression of type `{}`", expr_type.name()),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: self.assigned_span,
                    text: format!("expected type `{}`", assigned_type.name()),
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
    fn new(asg: &mut Asg, ctx: &mut AsgStatements, variable: &AstVarDefinition) -> Rc<Self> {
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

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        utils::result_ref(&self.expr)
            .map(AsgExpr::buffers)
            .unwrap_or_default()
    }
}

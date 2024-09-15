use crate::{asg, Asg, AsgBuffer, AsgExpr};
use fxhash::FxHashMap;
use shad_parser::{AstAssignment, AstIdent, AstRunItem, AstStatement};
use std::rc::Rc;

/// An analyzed compute shader definition.
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
    pub(crate) fn buffer_init(buffer: &Rc<AsgBuffer>) -> Self {
        let statements = vec![AsgStatement::Assignment(AsgAssignment {
            assigned: AsgValue::Buffer(buffer.clone()),
            value: buffer.expr.clone(),
        })];
        Self {
            buffers: Self::buffers(&statements),
            statements,
            name: format!("buffer_init:{}", buffer.name.label),
        }
    }

    pub(crate) fn step(asg: &mut Asg, ast_run: &AstRunItem) -> Self {
        let statements: Vec<_> = ast_run
            .statements
            .iter()
            .map(|statement| AsgStatement::new(asg, statement))
            .collect();
        Self {
            buffers: Self::buffers(&statements),
            statements,
            name: "run".into(),
        }
    }

    /// Returns all buffers used in the shader.
    fn buffers(statements: &[AsgStatement]) -> FxHashMap<String, Rc<AsgBuffer>> {
        statements.iter().flat_map(AsgStatement::buffers).collect()
    }
}

/// An analyzed statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgStatement {
    /// A variable assignment.
    Assignment(AsgAssignment),
}

impl AsgStatement {
    fn new(asg: &mut Asg, statement: &AstStatement) -> Self {
        match statement {
            AstStatement::Assignment(assigment) => {
                Self::Assignment(AsgAssignment::new(asg, assigment))
            }
        }
    }

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        match self {
            Self::Assignment(statement) => statement.buffers(),
        }
    }
}

/// An analyzed assignment statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgAssignment {
    /// A statement definition.
    pub assigned: AsgValue,
    /// The assigned value.
    pub value: AsgExpr,
}

impl AsgAssignment {
    fn new(asg: &mut Asg, assignment: &AstAssignment) -> Self {
        Self {
            assigned: AsgValue::new(asg, &assignment.value),
            value: AsgExpr::new(asg, &assignment.expr),
        }
    }

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        let mut buffers = self.assigned.buffers();
        buffers.extend(self.value.buffers());
        buffers
    }
}

/// An analyzed value definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgValue {
    /// An invalid value.
    Invalid,
    /// A buffer.
    Buffer(Rc<AsgBuffer>),
}

impl AsgValue {
    fn new(asg: &mut Asg, value: &AstIdent) -> Self {
        if let Some(buffer) = asg.buffers.get(&value.label) {
            Self::Buffer(buffer.clone())
        } else {
            asg.errors.push(asg::not_found_ident_error(asg, value));
            Self::Invalid
        }
    }

    fn buffers(&self) -> Vec<(String, Rc<AsgBuffer>)> {
        match self {
            // coverage: off (unreachable in `shad_runner` crate)
            Self::Invalid => vec![],
            // coverage: on
            Self::Buffer(buffer) => vec![(buffer.name.label.clone(), buffer.clone())],
        }
    }
}

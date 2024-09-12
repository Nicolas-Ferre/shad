use crate::{AsgBuffer, AsgExpr};
use std::iter;
use std::rc::Rc;

pub(crate) fn buffer_init_shader(buffer: &Rc<AsgBuffer>) -> AsgComputeShader {
    AsgComputeShader {
        buffers: iter::once(buffer.clone())
            .chain(buffer.expr.buffers())
            .collect(),
        statements: vec![AsgStatement::Assignment(AsgAssignment {
            assigned: AsgValue::Buffer(buffer.clone()),
            value: buffer.expr.clone(),
        })],
        name: format!("buffer_init:{}", buffer.name.label),
    }
}

/// An analyzed compute shader definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgComputeShader {
    /// The buffers to bind to the shader.
    pub buffers: Vec<Rc<AsgBuffer>>,
    /// The statements of the shader main function.
    pub statements: Vec<AsgStatement>,
    /// The name of the shader.
    pub name: String,
}

/// An analyzed statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgStatement {
    /// A variable assignment.
    Assignment(AsgAssignment),
}

/// An analyzed assignment statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgAssignment {
    /// A statement definition.
    pub assigned: AsgValue,
    /// The assigned value.
    pub value: AsgExpr,
}

/// An analyzed value definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgValue {
    /// A buffer.
    Buffer(Rc<AsgBuffer>),
}

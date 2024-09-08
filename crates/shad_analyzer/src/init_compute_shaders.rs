use crate::{AnalyzedBuffers, Buffer};
use std::rc::Rc;

/// All compute shaders run at startup generated from a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GeneratedInitComputeShaders {
    /// The generated shaders.
    pub shaders: Vec<ComputeShader>,
}

impl GeneratedInitComputeShaders {
    pub(crate) fn new(buffers: &AnalyzedBuffers) -> Self {
        let mut shaders = vec![];
        for buffer in buffers.buffers.values() {
            shaders.push(ComputeShader {
                buffers: vec![buffer.clone()],
                statements: vec![match &buffer.value {
                    shad_parser::Expr::Literal(literal) => Statement::Assignment(Assignment {
                        assigned: Value::Buffer(buffer.clone()),
                        value: Expr::Literal(literal.value.clone()),
                    }),
                }],
            });
        }
        Self { shaders }
    }
}

/// A compute shader definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ComputeShader {
    /// The buffers to bind to the shader.
    pub buffers: Vec<Rc<Buffer>>,
    /// The statements of the shader main function.
    pub statements: Vec<Statement>,
}

/// A statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    /// A variable assignment.
    Assignment(Assignment),
}

/// An assignment statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
    /// A statement definition.
    pub assigned: Value,
    /// The assigned value.
    pub value: Expr,
}

/// A value definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    /// A buffer.
    Buffer(Rc<Buffer>),
}

/// An expression definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    /// A literal.
    Literal(String),
}

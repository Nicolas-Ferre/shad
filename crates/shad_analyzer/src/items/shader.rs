use crate::{Asg, AsgAssignment, AsgBuffer, AsgFn, AsgStatement, BufferListing, FunctionListing};
use std::rc::Rc;

/// An analyzed compute shader.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgComputeShader {
    /// The buffers used in the shader.
    pub buffers: Vec<Rc<AsgBuffer>>,
    /// The functions used in the shader.
    pub functions: Vec<Rc<AsgFn>>,
    /// The statements of the shader main function.
    pub statements: Vec<AsgStatement>,
    /// The name of the shader.
    pub name: String,
}

impl AsgComputeShader {
    pub(crate) fn buffer_init(asg: &Asg, buffer: &Rc<AsgBuffer>) -> Self {
        let statements = vec![AsgStatement::Assignment(AsgAssignment::buffer_init(buffer))];
        Self {
            buffers: BufferListing::slice_buffers(&statements, asg),
            functions: FunctionListing::slice_functions(&statements, asg),
            statements,
            name: format!("buffer_init:{}", buffer.ast.name.label),
        }
    }

    pub(crate) fn step(asg: &Asg, statements: &[AsgStatement]) -> Self {
        Self {
            buffers: BufferListing::slice_buffers(statements, asg),
            functions: FunctionListing::slice_functions(statements, asg),
            statements: statements.to_vec(),
            name: "run".into(),
        }
    }
}

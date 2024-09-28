use crate::statement::{AsgAssignment, AsgStatement, AsgStatementScopeType, AsgStatements};
use crate::{Asg, AsgBuffer, AsgFn};
use fxhash::FxHashMap;
use shad_parser::AstRunItem;
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
    pub(crate) fn buffer_init(asg: &mut Asg, buffer: &Rc<AsgBuffer>) -> Self {
        let statements = vec![AsgStatement::Assignment(AsgAssignment::buffer_init(
            asg, buffer,
        ))];
        Self {
            buffers: Self::buffers(&statements, asg),
            functions: Self::functions(&statements, asg),
            statements,
            name: format!("buffer_init:{}", buffer.ast.name.label),
        }
    }

    pub(crate) fn step(asg: &mut Asg, ast_run: &AstRunItem) -> Self {
        let statements =
            AsgStatements::parse(asg, &ast_run.statements, AsgStatementScopeType::RunBlock);
        Self {
            buffers: Self::buffers(&statements, asg),
            functions: Self::functions(&statements, asg),
            statements,
            name: "run".into(),
        }
    }

    /// Returns all buffers used in the shader.
    fn buffers(statements: &[AsgStatement], asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        statements
            .iter()
            .flat_map(|statement| statement.buffers(asg))
            .map(|buffer| (buffer.index, buffer))
            .collect::<FxHashMap<_, _>>()
            .into_values()
            .collect()
    }

    /// Returns all functions used in the shader.
    fn functions(statements: &[AsgStatement], asg: &Asg) -> Vec<Rc<AsgFn>> {
        statements
            .iter()
            .flat_map(|statement| statement.functions(asg))
            .map(|buffer| (buffer.index, buffer))
            .collect::<FxHashMap<_, _>>()
            .into_values()
            .collect()
    }
}

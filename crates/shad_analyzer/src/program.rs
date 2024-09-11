use crate::{AnalyzedBuffers, AnalyzedTypes, GeneratedInitComputeShaders, SemanticError};
use shad_parser::Ast;

/// An analyzed Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedProgram {
    /// The analyzed types.
    pub types: AnalyzedTypes,
    /// The analyzed buffers.
    pub buffers: AnalyzedBuffers,
    /// The generated compute shaders run at program startup.
    pub init_compute_shaders: GeneratedInitComputeShaders,
}

impl AnalyzedProgram {
    /// Analyzes a Shad AST.
    pub fn analyze(ast: &Ast) -> Self {
        let types = AnalyzedTypes::new();
        let mut buffers = AnalyzedBuffers::default();
        buffers.init(ast, &types);
        let init_compute_shaders = GeneratedInitComputeShaders::new(ast, &buffers);
        Self {
            types,
            buffers,
            init_compute_shaders,
        }
    }

    /// Iterates on all semantic errors.
    pub fn errors(&self) -> impl Iterator<Item = &SemanticError> + '_ {
        self.buffers
            .errors
            .iter()
            .chain(&self.init_compute_shaders.errors)
    }
}

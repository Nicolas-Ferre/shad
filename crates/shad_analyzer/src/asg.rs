use crate::{buffer, shader, type_, AsgBuffer, AsgComputeShader, AsgType};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{Ast, AstItem};
use std::rc::Rc;

/// The Abstract Semantic Graph of an analyzed Shad code.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Asg {
    /// The raw Shad code.
    pub code: String,
    /// The path to the Shad code file.
    pub path: String,
    /// The analyzed types.
    pub types: FxHashMap<String, Rc<AsgType>>,
    /// The mapping between Shad buffer names and buffer index.
    pub buffers: FxHashMap<String, Rc<AsgBuffer>>,
    /// The initialization shaders.
    pub init_shaders: Vec<AsgComputeShader>,
    /// The semantic errors that occurred during the analysis.
    pub errors: Vec<SemanticError>,
}

#[allow(clippy::similar_names)]
impl Asg {
    /// Analyzes a Shad AST.
    pub fn analyze(ast: &Ast) -> Self {
        let mut asg = Self {
            code: ast.code.clone(),
            path: ast.path.clone(),
            types: FxHashMap::default(),
            buffers: FxHashMap::default(),
            init_shaders: vec![],
            errors: vec![],
        };
        asg.types = type_::primitive_types();
        Self::analyze_buffers(&mut asg, ast);
        asg
    }

    fn analyze_buffers(asg: &mut Self, ast: &Ast) {
        for item in &ast.items {
            let AstItem::Buffer(ast_buffer) = item;
            let buffer = Rc::new(buffer::analyze(asg, ast_buffer));
            asg.init_shaders.push(shader::buffer_init_shader(&buffer));
            let existing_buffer = asg.buffers.insert(buffer.name.label.clone(), buffer);
            if let Some(existing_buffer) = existing_buffer {
                asg.errors.push(buffer::duplicated_name_error(
                    asg,
                    ast_buffer,
                    &existing_buffer,
                ));
            }
        }
    }
}

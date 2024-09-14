use crate::{buffer, function, type_, AsgBuffer, AsgComputeShader, AsgFn, AsgFnSignature, AsgType};
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
    /// The analyzed functions.
    pub functions: FxHashMap<AsgFnSignature, Rc<AsgFn>>,
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
            functions: FxHashMap::default(),
            buffers: FxHashMap::default(),
            init_shaders: vec![],
            errors: vec![],
        };
        asg.types = type_::primitive_types();
        Self::analyze_functions(&mut asg, ast);
        Self::analyze_buffers(&mut asg, ast);
        asg
    }

    fn analyze_functions(asg: &mut Self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::GpuFn(ast_fn) = item {
                let asg_fn = AsgFn::new(asg, ast_fn);
                let fn_ = Rc::new(asg_fn);
                let signature = AsgFnSignature::new(&fn_);
                if let Some(existing_fn) = asg.functions.insert(signature, fn_) {
                    asg.errors
                        .push(function::duplicated_error(asg, ast_fn, &existing_fn));
                }
            }
        }
    }

    fn analyze_buffers(asg: &mut Self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Buffer(ast_buffer) = item {
                let name = ast_buffer.name.label.clone();
                let buffer = Rc::new(AsgBuffer::new(asg, ast_buffer));
                let init_shader = AsgComputeShader::buffer_init(&buffer);
                asg.init_shaders.push(init_shader);
                if let Some(existing_buffer) = asg.buffers.insert(name, buffer) {
                    asg.errors
                        .push(buffer::duplicated_error(asg, ast_buffer, &existing_buffer));
                }
            }
        }
    }
}

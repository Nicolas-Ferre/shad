use crate::passes::recursion_check::check_recursion;
use crate::statement::AsgStatements;
use crate::{
    errors, type_, AsgBuffer, AsgComputeShader, AsgFn, AsgFnBody, AsgFnSignature, AsgType,
};
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
    /// The analyzed function bodies.
    pub function_bodies: FxHashMap<AsgFnSignature, AsgFnBody>,
    /// The mapping between Shad buffer names and buffer index.
    pub buffers: FxHashMap<String, Rc<AsgBuffer>>,
    /// The initialization shaders.
    pub init_shaders: Vec<AsgComputeShader>,
    /// The shaders run during each step of the application loop.
    pub step_shaders: Vec<AsgComputeShader>,
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
            function_bodies: FxHashMap::default(),
            buffers: FxHashMap::default(),
            init_shaders: vec![],
            step_shaders: vec![],
            errors: vec![],
        };
        asg.types = type_::primitive_types();
        asg.register_functions(ast);
        asg.register_buffers(ast);
        asg.register_function_bodies();
        asg.errors.extend(check_recursion(&asg));
        asg.register_init_shaders();
        asg.register_step_shaders(ast);
        asg
    }

    fn register_functions(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Fn(ast_fn) = item {
                let asg_fn = AsgFn::new(self, ast_fn);
                let fn_ = Rc::new(asg_fn);
                let signature = fn_.signature.clone();
                if let Some(existing_fn) = self.functions.insert(signature, fn_) {
                    self.errors
                        .push(errors::fn_::duplicated(self, ast_fn, &existing_fn));
                }
            }
        }
    }

    fn register_buffers(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Buffer(ast_buffer) = item {
                let name = ast_buffer.name.label.clone();
                let statements = AsgStatements::buffer_scope();
                let buffer = Rc::new(AsgBuffer::new(self, &statements, ast_buffer));
                if let Some(existing_buffer) = self.buffers.insert(name, buffer) {
                    self.errors.push(errors::buffer::duplicated(
                        self,
                        ast_buffer,
                        &existing_buffer,
                    ));
                }
            }
        }
    }

    fn register_function_bodies(&mut self) {
        for (signature, fn_) in self.functions.clone() {
            let body = AsgFnBody::new(self, &fn_);
            self.function_bodies.insert(signature, body);
        }
    }

    fn register_init_shaders(&mut self) {
        let mut buffers: Vec<_> = self.buffers.values().cloned().collect();
        buffers.sort_unstable_by_key(|buffer| buffer.index);
        for buffer in buffers {
            let init_shader = AsgComputeShader::buffer_init(self, &buffer);
            self.init_shaders.push(init_shader);
        }
    }

    fn register_step_shaders(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Run(ast_run) = item {
                let shader = AsgComputeShader::step(self, ast_run);
                self.step_shaders.push(shader);
            }
        }
    }
}

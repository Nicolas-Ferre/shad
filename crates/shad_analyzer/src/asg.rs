use crate::items::statement::StatementContext;
use crate::items::type_;
use crate::passes::check::{check, StatementScope};
use crate::passes::fn_inlining::inline_fns;
use crate::passes::fn_param_extraction::extract_fn_params;
use crate::passes::recursion_check::check_recursion;
use crate::{
    errors, AsgAssignment, AsgBuffer, AsgComputeShader, AsgFn, AsgFnBody, AsgFnSignature,
    AsgStatement, AsgType, Error, Result,
};
use fxhash::FxHashMap;
use shad_error::{SemanticError, Span};
use shad_parser::{Ast, AstIdent, AstItem};
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
    /// The buffer init statements.
    pub buffer_inits: Vec<Vec<AsgStatement>>,
    /// The `run` block statements.
    pub run_blocks: Vec<Vec<AsgStatement>>,
    /// The initialization shaders.
    pub init_shaders: Vec<AsgComputeShader>,
    /// The shaders run during each step of the application loop.
    pub step_shaders: Vec<AsgComputeShader>,
    /// The semantic errors that occurred during the analysis.
    pub errors: Vec<SemanticError>,
    var_next_index: usize,
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
            buffer_inits: vec![],
            run_blocks: vec![],
            init_shaders: vec![],
            step_shaders: vec![],
            errors: vec![],
            var_next_index: 0,
        };
        asg.types = type_::primitive_types();
        asg.register_functions(ast);
        asg.register_buffers(ast);
        asg.register_function_bodies();
        asg.register_buffer_inits();
        asg.register_run_blocks(ast);
        asg.errors.extend(check_recursion(&asg));
        asg.errors.extend(check(&asg));
        extract_fn_params(&mut asg);
        if asg.errors.is_empty() {
            inline_fns(&mut asg);
        }
        asg.register_init_shaders();
        asg.register_step_shaders();
        asg
    }

    pub(crate) fn next_var_index(&mut self) -> usize {
        let index = self.var_next_index;
        self.var_next_index += 1;
        index
    }

    pub(crate) fn find_type(&mut self, name: &AstIdent) -> Result<&Rc<AsgType>> {
        if let Some(type_) = self.types.get(&name.label) {
            Ok(type_)
        } else {
            self.errors.push(errors::type_::not_found(self, name));
            Err(Error)
        }
    }

    pub(crate) fn find_function(
        &mut self,
        span: Span,
        signature: &AsgFnSignature,
    ) -> Result<&Rc<AsgFn>> {
        if let Some(function) = self.functions.get(signature) {
            Ok(function)
        } else {
            self.errors
                .push(errors::fn_::not_found(self, span, signature));
            Err(Error)
        }
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
                let buffer = Rc::new(AsgBuffer::new(self, ast_buffer));
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

    fn register_buffer_inits(&mut self) {
        let mut buffers: Vec<_> = self.buffers.values().cloned().collect();
        buffers.sort_unstable_by_key(|buffer| buffer.index);
        for buffer in buffers {
            self.buffer_inits
                .push(vec![AsgStatement::Assignment(AsgAssignment::buffer_init(
                    &buffer,
                ))]);
        }
    }

    fn register_run_blocks(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Run(ast_run) = item {
                let statements =
                    StatementContext::analyze(self, &ast_run.statements, StatementScope::RunBlock);
                self.run_blocks.push(statements);
            }
        }
    }

    fn register_init_shaders(&mut self) {
        for statements in &self.buffer_inits {
            let shader = AsgComputeShader::new(self, statements);
            self.init_shaders.push(shader);
        }
    }

    fn register_step_shaders(&mut self) {
        for statements in &self.run_blocks {
            let shader = AsgComputeShader::new(self, statements);
            self.step_shaders.push(shader);
        }
    }
}

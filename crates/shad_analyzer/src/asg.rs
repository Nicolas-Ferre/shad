use crate::function::FnRecursionChecker;
use crate::statement::AsgStatements;
use crate::{
    buffer, function, type_, AsgBuffer, AsgComputeShader, AsgFn, AsgFnBody, AsgFnSignature, AsgType,
};
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
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
        asg.analyze_functions(ast);
        asg.analyze_buffers(ast);
        asg.analyze_function_bodies();
        asg.analyze_init_shaders();
        asg.analyze_step_shaders(ast);
        asg
    }

    fn analyze_functions(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Fn(ast_fn) = item {
                let asg_fn = AsgFn::new(self, ast_fn);
                let fn_ = Rc::new(asg_fn);
                let signature = fn_.signature.clone();
                if let Some(existing_fn) = self.functions.insert(signature, fn_) {
                    self.errors
                        .push(function::duplicated_error(self, ast_fn, &existing_fn));
                }
            }
        }
    }

    fn analyze_buffers(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Buffer(ast_buffer) = item {
                let name = ast_buffer.name.label.clone();
                let statements = AsgStatements::buffer_scope();
                let buffer = Rc::new(AsgBuffer::new(self, &statements, ast_buffer));
                if let Some(existing_buffer) = self.buffers.insert(name, buffer) {
                    self.errors
                        .push(buffer::duplicated_error(self, ast_buffer, &existing_buffer));
                }
            }
        }
    }

    fn analyze_function_bodies(&mut self) {
        for (signature, fn_) in self.functions.clone() {
            let body = AsgFnBody::new(self, &fn_);
            self.function_bodies.insert(signature, body);
        }
        let mut checker = FnRecursionChecker::default();
        for fn_ in self.functions.values() {
            checker.current_fn = Some(fn_.clone());
            checker.calls.clear();
            let _ = fn_.check_recursion(self, &mut checker);
        }
        self.errors.extend(checker.errors);
    }

    fn analyze_init_shaders(&mut self) {
        let mut buffers: Vec<_> = self.buffers.values().cloned().collect();
        buffers.sort_unstable_by_key(|buffer| buffer.index);
        for buffer in buffers {
            let init_shader = AsgComputeShader::buffer_init(self, &buffer);
            self.init_shaders.push(init_shader);
        }
    }

    fn analyze_step_shaders(&mut self, ast: &Ast) {
        for item in &ast.items {
            if let AstItem::Run(ast_run) = item {
                let shader = AsgComputeShader::step(self, ast_run);
                self.step_shaders.push(shader);
            }
        }
    }
}

pub(crate) fn not_found_ident_error(asg: &Asg, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` value", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined identifier".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

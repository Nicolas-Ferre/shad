use crate::registration::functions::Function;
use crate::registration::idents::Ident;
use crate::registration::shaders::ComputeShader;
use crate::{
    checks, registration, transformation, Buffer, IdentSource, RunBlock, Type, BOOL_TYPE, F32_TYPE,
    I32_TYPE, U32_TYPE,
};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{Ast, AstExpr, AstIdent, AstLiteralType};

/// The semantic analysis of an AST.
#[derive(Debug, Clone)]
pub struct Analysis {
    /// The AST.
    pub ast: Ast,
    /// The analyzed identifiers.
    pub idents: FxHashMap<u64, Ident>,
    /// The analyzed types.
    pub types: FxHashMap<String, Type>,
    /// The analyzed functions.
    pub fns: FxHashMap<String, Function>,
    /// The analyzed buffers.
    pub buffers: FxHashMap<String, Buffer>,
    /// The analyzed init blocks.
    pub init_blocks: Vec<RunBlock>,
    /// The analyzed run blocks.
    pub run_blocks: Vec<RunBlock>,
    /// The analyzed init shaders.
    pub init_shaders: Vec<ComputeShader>,
    /// The analyzed step shaders.
    pub step_shaders: Vec<ComputeShader>,
    /// The semantic errors found during analysis.
    pub errors: Vec<SemanticError>,
}

impl Analysis {
    /// Runs the semantic analysis on an `ast`.
    pub fn run(ast: &Ast) -> Self {
        let mut analysis = Self {
            ast: ast.clone(),
            idents: FxHashMap::default(),
            types: FxHashMap::default(),
            fns: FxHashMap::default(),
            buffers: FxHashMap::default(),
            init_blocks: vec![],
            run_blocks: vec![],
            init_shaders: vec![],
            step_shaders: vec![],
            errors: vec![],
        };
        registration::types::register(&mut analysis);
        registration::functions::register(&mut analysis);
        registration::buffers::register(&mut analysis);
        registration::run_blocks::register(&mut analysis);
        transformation::literals::transform(&mut analysis);
        transformation::fn_params::transform(&mut analysis);
        registration::idents::register(&mut analysis);
        checks::functions::check(&mut analysis);
        checks::literals::check(&mut analysis);
        checks::statements::check(&mut analysis);
        checks::fn_recursion::check(&mut analysis);
        if analysis.errors.is_empty() {
            transformation::ref_split::transform(&mut analysis);
            transformation::ref_fn_inline::transform(&mut analysis);
            transformation::ref_var_inline::transform(&mut analysis);
            registration::shaders::register(&mut analysis);
        }
        analysis
    }

    /// Returns the type of a buffer.
    pub fn buffer_type(&self, buffer_name: &str) -> &Type {
        let id = &self.buffers[buffer_name].ast.name.id;
        let type_ = self.idents[id]
            .type_
            .as_deref()
            .expect("internal error: invalid buffer type");
        &self.types[type_]
    }

    pub(crate) fn expr_type(&self, expr: &AstExpr) -> Option<String> {
        match expr {
            AstExpr::Literal(literal) => Some(match literal.type_ {
                AstLiteralType::F32 => F32_TYPE.into(),
                AstLiteralType::U32 => U32_TYPE.into(),
                AstLiteralType::I32 => I32_TYPE.into(),
                AstLiteralType::Bool => BOOL_TYPE.into(),
            }),
            AstExpr::Ident(ident) => self.idents.get(&ident.id)?.type_.clone(),
            AstExpr::FnCall(call) => self.idents.get(&call.name.id)?.type_.clone(),
        }
    }

    pub(crate) fn fn_signature(&self, fn_name: &AstIdent) -> Option<String> {
        self.idents
            .get(&fn_name.id)
            .map(|ident| match &ident.source {
                IdentSource::Fn(signature) => signature.clone(),
                IdentSource::Buffer(_) | IdentSource::Var(_) => {
                    unreachable!("internal error: retrieve non-function signature")
                }
            })
    }
}

use crate::registration::const_functions::{ConstFn, ConstFnId};
use crate::registration::constants::{Constant, ConstantId};
use crate::registration::functions::Function;
use crate::registration::idents::Ident;
use crate::registration::shaders::ComputeShader;
use crate::{
    checks, registration, resolver, transformation, Buffer, BufferId, BufferInitRunBlock, FnId,
    RunBlock, Type, TypeId,
};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{Ast, AstIdent};

/// The semantic analysis of an AST.
#[derive(Debug, Clone)]
pub struct Analysis {
    /// The module ASTs.
    pub asts: FxHashMap<String, Ast>,
    /// From each module, the list of visible modules sorted by priority.
    pub const_functions: FxHashMap<ConstFnId, ConstFn>,
    /// The analyzed identifiers.
    pub visible_modules: FxHashMap<String, Vec<String>>,
    /// The builtin constant functions.
    pub idents: FxHashMap<u64, Ident>,
    /// The analyzed types.
    pub types: FxHashMap<TypeId, Type>,
    /// The analyzed functions.
    pub fns: FxHashMap<FnId, Function>,
    /// The analyzed constants.
    pub constants: FxHashMap<ConstantId, Constant>,
    /// The analyzed buffers.
    pub buffers: FxHashMap<BufferId, Buffer>,
    /// The analyzed init blocks.
    pub init_blocks: Vec<BufferInitRunBlock>,
    /// The analyzed run blocks.
    pub run_blocks: Vec<RunBlock>,
    /// The analyzed init shaders.
    pub init_shaders: Vec<ComputeShader>,
    /// The analyzed step shaders.
    pub step_shaders: Vec<ComputeShader>,
    /// The semantic errors found during analysis.
    pub errors: Vec<SemanticError>,
    next_id: u64,
}

impl Analysis {
    /// Runs the semantic analysis on an `ast`.
    pub fn run(asts: FxHashMap<String, Ast>) -> Self {
        let next_id = asts.values().map(|ast| ast.next_id).max().unwrap_or(0);
        let mut analysis = Self {
            asts,
            const_functions: FxHashMap::default(),
            visible_modules: FxHashMap::default(),
            idents: FxHashMap::default(),
            types: FxHashMap::default(),
            fns: FxHashMap::default(),
            constants: FxHashMap::default(),
            buffers: FxHashMap::default(),
            init_blocks: vec![],
            run_blocks: vec![],
            init_shaders: vec![],
            step_shaders: vec![],
            errors: vec![],
            next_id,
        };
        registration::const_functions::register(&mut analysis);
        registration::modules::register(&mut analysis);
        registration::types::register(&mut analysis);
        registration::functions::register(&mut analysis);
        registration::constants::register(&mut analysis);
        registration::buffers::register(&mut analysis);
        registration::run_blocks::register(&mut analysis);
        transformation::fn_params::transform(&mut analysis);
        registration::idents::register(&mut analysis);
        transformation::literals::transform(&mut analysis);
        registration::constants::calculate(&mut analysis);
        checks::constants::check(&mut analysis);
        checks::generics::check(&mut analysis);
        checks::functions::check(&mut analysis);
        checks::types::check(&mut analysis);
        checks::literals::check(&mut analysis);
        checks::statements::check(&mut analysis);
        checks::recursion::fns::check(&mut analysis);
        if !analysis.errors.is_empty() {
            return analysis;
        }
        checks::recursion::constants::check(&mut analysis);
        checks::recursion::buffers::check(&mut analysis);
        if !analysis.errors.is_empty() {
            return analysis;
        }
        checks::recursion::types::check(&mut analysis);
        transformation::constants::transform(&mut analysis);
        transformation::left_values::transform(&mut analysis);
        transformation::ref_split::transform(&mut analysis);
        transformation::ref_fn_inline::transform(&mut analysis);
        transformation::ref_var_inline::transform(&mut analysis);
        transformation::expr_statements::transform(&mut analysis);
        registration::shaders::register(&mut analysis);
        analysis
    }

    /// Returns the type of a buffer.
    pub fn buffer_type(&self, buffer_id: &BufferId) -> Option<&Type> {
        resolver::buffer_type(self, buffer_id)
    }

    /// Returns the function from a function name identifier.
    pub fn fn_(&self, ident: &AstIdent) -> Option<&Function> {
        resolver::fn_(self, ident)
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let next_id = self.next_id;
        self.next_id += 1;
        next_id
    }
}

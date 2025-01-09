use crate::registration::const_functions::{ConstFn, ConstFnId};
use crate::registration::constants::{Constant, ConstantId};
use crate::registration::functions::Function;
use crate::registration::idents::Ident;
use crate::registration::shaders::ComputeShader;
use crate::{
    checks, registration, resolving, transformation, Buffer, BufferId, BufferInitRunBlock, FnId,
    IdentSource, RunBlock, Type, TypeId,
};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{Ast, AstFnCall, AstIdent};

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
        registration::run_blocks::register(&mut analysis);
        registration::const_functions::register(&mut analysis);
        registration::modules::register(&mut analysis);
        registration::types::register(&mut analysis);
        registration::functions::register(&mut analysis);
        registration::constants::register(&mut analysis);
        registration::buffers::register(&mut analysis);
        transformation::fn_params::transform(&mut analysis);
        registration::idents::register(&mut analysis);
        checks::constants::check(&mut analysis);
        checks::generics::check(&mut analysis);
        checks::functions::check(&mut analysis);
        checks::gpu_names::check(&mut analysis);
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

    /// Returns the item corresponding to an identifier.
    pub fn item(&self, ident: &AstIdent) -> Option<Item<'_>> {
        if let Some(ident) = self.idents.get(&ident.id) {
            let IdentSource::Var(id) = &ident.source;
            self.idents
                .get(id)
                .map(|ident| Item::Var(*id, &ident.type_id))
        } else {
            resolving::items::buffer(self, ident)
                .map(Item::Buffer)
                .or_else(|| resolving::items::constant(self, ident).map(Item::Constant))
        }
    }

    /// Returns the type of a buffer.
    pub fn buffer_type(&self, buffer_id: &BufferId) -> Option<&Type> {
        resolving::types::buffer(self, buffer_id)
    }

    /// Returns the function corresponding to a function call.
    pub fn fn_(&self, call: &AstFnCall) -> Option<&Function> {
        resolving::items::fn_(self, call)
    }

    /// Returns the type ID corresponding to an identifier.
    pub fn type_id(&self, ident: &AstIdent) -> Option<TypeId> {
        resolving::items::type_id(self, ident).ok()
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let next_id = self.next_id;
        self.next_id += 1;
        next_id
    }
}

/// A found item.
#[derive(Debug, Clone)]
pub enum Item<'a> {
    /// A constant.
    Constant(&'a Constant),
    /// A buffer.
    Buffer(&'a Buffer),
    /// A variable.
    Var(u64, &'a Option<TypeId>),
}

use crate::{resolving, Analysis, TypeId};
use fxhash::FxHashMap;
use shad_parser::{AstExpr, AstExprRoot, AstFnItem, AstFnParam, AstIdent, AstVarDefinition, Visit};
use std::mem;

/// An analyzed identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    /// The source of the identifier.
    pub source: IdentSource,
    /// The type ID of the identifier.
    pub type_id: Option<TypeId>,
}

impl Ident {
    pub(crate) fn new(source: IdentSource, type_id: Option<TypeId>) -> Self {
        Self { source, type_id }
    }
}

/// The source of an identifier.
#[derive(Debug, Clone)]
pub enum IdentSource {
    /// A variable.
    Var(u64),
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_constants(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_constants(analysis: &mut Analysis) {
    for constant in analysis.constants.clone().values() {
        IdentRegistration::new(analysis, true).visit_const_item(&constant.ast);
    }
}

fn register_run_blocks(analysis: &mut Analysis) {
    let blocks = mem::take(&mut analysis.run_blocks);
    for block in &blocks {
        IdentRegistration::new(analysis, false).visit_run_item(&block.ast);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    for fn_ in analysis.fns.clone().into_values() {
        IdentRegistration::new(analysis, false).visit_fn_item(&fn_.ast);
    }
}

struct IdentRegistration<'a> {
    analysis: &'a mut Analysis,
    is_const_context: bool,
    variables: FxHashMap<String, u64>,
}

impl<'a> IdentRegistration<'a> {
    pub(crate) fn new(analysis: &'a mut Analysis, is_const_context: bool) -> Self {
        Self {
            analysis,
            is_const_context,
            variables: FxHashMap::default(),
        }
    }

    fn add_variable(&mut self, ident: &AstIdent) {
        self.variables.insert(ident.label.clone(), ident.id);
    }

    fn register_fn_param(&mut self, param: &AstFnParam) {
        let type_id = resolving::items::type_id(self.analysis, &param.type_).ok();
        let ident = Ident::new(IdentSource::Var(param.name.id), type_id);
        self.analysis.idents.insert(param.name.id, ident);
        self.add_variable(&param.name);
    }
}

impl Visit for IdentRegistration<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        for param in &node.params {
            self.register_fn_param(param);
        }
    }

    fn exit_var_definition(&mut self, node: &AstVarDefinition) {
        let var_type = resolving::types::expr(self.analysis, &node.expr);
        let var_ident = Ident::new(IdentSource::Var(node.name.id), var_type);
        self.analysis.idents.insert(node.name.id, var_ident);
        self.add_variable(&node.name);
    }

    fn exit_expr(&mut self, node: &AstExpr) {
        if self.is_const_context {
            return;
        }
        if let AstExprRoot::Ident(value) = &node.root {
            if let Some(&id) = self.variables.get(&value.label) {
                let var_type = self
                    .analysis
                    .idents
                    .get(&id)
                    .and_then(|var| var.type_id.clone());
                let var_ident = Ident::new(IdentSource::Var(id), var_type);
                self.analysis.idents.insert(value.id, var_ident);
            }
        }
    }
}

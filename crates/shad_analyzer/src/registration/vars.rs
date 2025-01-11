use crate::{resolving, Analysis, Function, TypeId};
use fxhash::FxHashMap;
use shad_parser::{AstIdent, AstIdentKind, AstVarDefinition, VisitMut};
use std::mem;

/// An analyzed local variable.
#[derive(Debug, Clone)]
pub struct Var {
    /// The ID of the variable type.
    pub(crate) type_id: Option<TypeId>,
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init_blocks(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_init_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.init_blocks);
    for block in &mut blocks {
        VarRegistration::new(analysis).visit_run_item(&mut block.ast);
    }
    analysis.init_blocks = blocks;
}

fn register_run_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        VarRegistration::new(analysis).visit_run_item(&mut block.ast);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    let mut fns = analysis.fns.clone();
    for fn_ in fns.values_mut() {
        register_fn(analysis, fn_);
    }
    analysis.fns = fns;
}

pub(crate) fn register_fn(analysis: &mut Analysis, fn_: &mut Function) {
    let mut registration = VarRegistration::new(analysis);
    for (param, param_ast) in fn_.params.iter_mut().zip(&mut fn_.ast.params) {
        registration.register_var(&mut param.name, param.type_id.clone());
        param_ast.name.var_id = param.name.var_id;
    }
    for statement in &mut fn_.ast.statements {
        registration.visit_statement(statement);
    }
}

struct VarRegistration<'a> {
    analysis: &'a mut Analysis,
    var_ids: FxHashMap<String, u64>,
}

impl<'a> VarRegistration<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            var_ids: FxHashMap::default(),
        }
    }

    fn register_var(&mut self, node: &mut AstIdent, type_id: Option<TypeId>) {
        node.var_id = self.analysis.next_id();
        self.var_ids.insert(node.label.clone(), node.var_id);
        self.analysis.vars.insert(node.var_id, Var { type_id });
    }
}

impl VisitMut for VarRegistration<'_> {
    fn exit_var_definition(&mut self, node: &mut AstVarDefinition) {
        self.register_var(
            &mut node.name,
            resolving::types::expr(self.analysis, &node.expr),
        );
    }

    fn exit_ident(&mut self, node: &mut AstIdent) {
        match node.kind {
            AstIdentKind::Other => {
                if let Some(id) = self.var_ids.get(&node.label) {
                    node.var_id = *id;
                }
            }
            AstIdentKind::VarDef | AstIdentKind::FnRef | AstIdentKind::FieldRef => (),
        }
    }
}

use crate::{resolving, Analysis, Function};
use shad_parser::{AstFnCall, AstItemGenerics, AstStatement, Visit};
use std::mem;

pub(crate) fn register(analysis: &mut Analysis) {
    analysis.raw_fns.clone_from(&analysis.fns);
    register_init_blocks(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_init_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.init_blocks);
    for block in &mut blocks {
        register_statements(analysis, &block.ast.statements);
    }
    analysis.init_blocks = blocks;
}

fn register_run_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        register_statements(analysis, &block.ast.statements);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    let mut fns = analysis.fns.clone();
    for fn_ in fns.values_mut() {
        if fn_.generics.is_empty() {
            register_statements(analysis, &fn_.ast.statements);
        }
    }
}

fn register_statements(analysis: &mut Analysis, statements: &[AstStatement]) {
    for statement in statements {
        GenericFnRegistration::new(analysis).visit_statement(statement);
    }
}

struct GenericFnRegistration<'a> {
    analysis: &'a mut Analysis,
}

impl<'a> GenericFnRegistration<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self { analysis }
    }

    fn specialize_fn(&self, fn_: &Function, call: &AstFnCall) -> Option<Function> {
        let mut specialized_fn = fn_.clone();
        specialized_fn.ast.generics = AstItemGenerics {
            span: fn_.ast.generics.span.clone(),
            params: vec![],
        };
        specialized_fn.generics = vec![];
        specialized_fn.id.is_generic = false;
        specialized_fn.id.param_types = resolving::types::fn_args(self.analysis, call)?
            .into_iter()
            .map(Some)
            .collect();
        specialized_fn.id.generic_values =
            resolving::expressions::fn_call_generic_values(self.analysis, call)?;
        Some(specialized_fn)
    }
}

impl Visit for GenericFnRegistration<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolving::items::fn_(self.analysis, node, false) {
            if !fn_.generics.is_empty() {
                if let Some(specialized_fn) = self.specialize_fn(fn_, node) {
                    let statements = specialized_fn.ast.statements.clone();
                    self.analysis
                        .fns
                        .insert(specialized_fn.id.clone(), specialized_fn);
                    register_statements(self.analysis, &statements);
                }
            }
        }
    }
}

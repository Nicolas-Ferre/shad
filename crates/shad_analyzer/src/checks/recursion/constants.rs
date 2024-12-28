use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::registration::constants::ConstantId;
use crate::{errors, resolver, Analysis, IdentSource};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstIdent, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_constant_ids = FxHashSet::default();
    for constant in analysis.constants.values() {
        let mut checker = ItemRecursionCheck::new(
            analysis,
            ConstantId::new(&constant.ast),
            mem::take(&mut errored_constant_ids),
        );
        checker.visit_expr(&constant.ast.value);
        errors.extend(checker.errors);
        errored_constant_ids = checker.errored_item_ids;
    }
    analysis.errors.extend(errors);
}

impl Visit for ItemRecursionCheck<'_, ConstantId> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolver::fn_(self.analysis, &node.name) {
            self.visit_fn_item(&fn_.ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        if let Some(ident) = self.analysis.idents.get(&node.id) {
            if let IdentSource::Constant(id) = &ident.source {
                let constant = &self.analysis.constants[id].ast;
                self.used_item_ids.push(UsedItem {
                    usage_span: node.span.clone(),
                    def_span: constant.name.span.clone(),
                    id: id.clone(),
                });
                if !self.detect_error(errors::constants::recursion_found) {
                    self.visit_expr(&constant.value);
                }
                self.used_item_ids.pop();
            }
        }
    }
}

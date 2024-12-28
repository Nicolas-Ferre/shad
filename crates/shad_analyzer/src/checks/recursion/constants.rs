use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::registration::constants::ConstantId;
use crate::{errors, Analysis, IdentSource};
use fxhash::FxHashSet;
use shad_parser::{AstIdent, Visit};
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
    fn enter_ident(&mut self, node: &AstIdent) {
        let ident = &self.analysis.idents[&node.id];
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
        } else {
            unreachable!("internal error: non-constant identifier in const context")
        }
    }
}

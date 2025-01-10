use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::registration::constants::ConstantId;
use crate::{errors, Analysis, Item};
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
        if let Some(Item::Constant(constant)) = self.analysis.item(node) {
            self.used_item_ids.push(UsedItem {
                usage_span: node.span.clone(),
                def_span: constant.ast.name.span.clone(),
                id: constant.id.clone(),
                name: constant.ast.name.label.clone(),
            });
            if !self.detect_error(|_analysis, type_id, type_stack| {
                errors::constants::recursion_found(type_id, type_stack)
            }) {
                self.visit_expr(&constant.ast.value);
            }
            self.used_item_ids.pop();
        }
    }
}

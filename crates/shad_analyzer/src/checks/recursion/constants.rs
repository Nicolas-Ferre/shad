use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::registration::constants::ConstantId;
use crate::resolving::items::Item;
use crate::{errors, resolving, Analysis};
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
        if let Some(Item::Constant(constant)) = resolving::items::item(self.analysis, node) {
            self.used_item_ids.push(UsedItem {
                usage_span: node.span.clone(),
                def_span: constant.ast.name.span.clone(),
                id: constant.id.clone(),
            });
            if !self.detect_error(errors::constants::recursion_found) {
                self.visit_expr(&constant.ast.value);
            }
            self.used_item_ids.pop();
        }
    }
}

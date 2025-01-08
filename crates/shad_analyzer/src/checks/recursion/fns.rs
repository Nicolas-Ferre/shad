use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::{errors, resolving, Analysis, FnId};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut all_errored_fn_ids = FxHashSet::default();
    for (fn_id, fn_) in &analysis.fns {
        let errored_fn_ids = mem::take(&mut all_errored_fn_ids);
        let mut checker = ItemRecursionCheck::new(analysis, fn_id.clone(), errored_fn_ids);
        checker.visit_fn_item(&fn_.ast);
        errors.extend(checker.errors);
        all_errored_fn_ids = checker.errored_item_ids;
    }
    analysis.errors.extend(errors);
}

impl Visit for ItemRecursionCheck<'_, FnId> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolving::items::registered_fn(self.analysis, &node.name) {
            self.used_item_ids.push(UsedItem {
                usage_span: node.span.clone(),
                def_span: fn_.ast.name.span.clone(),
                id: fn_.id.clone(),
            });
            if !self.detect_error(errors::functions::recursion_found) {
                self.visit_fn_item(&fn_.ast);
            }
            self.used_item_ids.pop();
        }
    }
}

use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::resolving::items::Item;
use crate::{errors, resolving, Analysis, BufferId};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstIdent, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_buffer_ids = FxHashSet::default();
    for buffer in analysis.buffers.values() {
        let mut checker = ItemRecursionCheck::new(
            analysis,
            BufferId::new(&buffer.ast),
            mem::take(&mut errored_buffer_ids),
        );
        checker.visit_expr(&buffer.ast.value);
        errors.extend(checker.errors);
        errored_buffer_ids = checker.errored_item_ids;
    }
    analysis.errors.extend(errors);
}

impl Visit for ItemRecursionCheck<'_, BufferId> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(Item::Fn(fn_)) = resolving::items::item(self.analysis, &node.name) {
            self.visit_fn_item(&fn_.ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        if let Some(Item::Buffer(buffer)) = resolving::items::item(self.analysis, node) {
            self.used_item_ids.push(UsedItem {
                usage_span: node.span.clone(),
                def_span: buffer.ast.name.span.clone(),
                id: buffer.id.clone(),
            });
            if !self.detect_error(errors::buffers::recursion_found) {
                self.visit_expr(&buffer.ast.value);
            }
            self.used_item_ids.pop();
        }
    }
}

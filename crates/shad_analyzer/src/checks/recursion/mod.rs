pub(crate) mod buffers;
pub(crate) mod fns;
pub(crate) mod types;

use crate::Analysis;
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use std::hash::Hash;

#[derive(Debug)]
pub(crate) struct UsedItem<I> {
    pub(crate) usage_span: Span,
    pub(crate) def_span: Span,
    pub(crate) id: I,
}

struct ItemRecursionCheck<'a, I> {
    analysis: &'a Analysis,
    current_item_id: I,
    used_item_ids: Vec<UsedItem<I>>,
    errored_item_ids: FxHashSet<I>,
    errors: Vec<SemanticError>,
}

impl<'a, I> ItemRecursionCheck<'a, I>
where
    I: Clone + Eq + Hash,
{
    fn new(analysis: &'a Analysis, item_id: I, errored_item_ids: FxHashSet<I>) -> Self {
        Self {
            analysis,
            current_item_id: item_id,
            used_item_ids: vec![],
            errored_item_ids,
            errors: vec![],
        }
    }

    fn detect_error(&mut self, error: fn(&I, &[UsedItem<I>]) -> SemanticError) -> bool {
        if !self.is_last_usage_recursive() {
            false
        } else if self.is_error_already_generated() {
            true
        } else {
            for call in &self.used_item_ids {
                self.errored_item_ids.insert(call.id.clone());
            }
            self.errored_item_ids.insert(self.current_item_id.clone());
            self.errors
                .push(error(&self.current_item_id, &self.used_item_ids));
            true
        }
    }

    fn is_last_usage_recursive(&self) -> bool {
        self.used_item_ids
            .last()
            .map_or(false, |last_call| last_call.id == self.current_item_id)
    }

    fn is_error_already_generated(&self) -> bool {
        for call in &self.used_item_ids {
            if self.errored_item_ids.contains(&call.id) {
                return true;
            }
        }
        self.errored_item_ids.contains(&self.current_item_id)
    }
}

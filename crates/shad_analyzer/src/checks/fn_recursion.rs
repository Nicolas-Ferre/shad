use crate::{errors, search, Analysis, FnId};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use shad_parser::{AstFnCall, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut all_errored_fn_ids = FxHashSet::default();
    for (fn_id, fn_) in &analysis.fns {
        let errored_fn_ids = mem::take(&mut all_errored_fn_ids);
        let mut checker = FnRecursionCheck::new(analysis, fn_id.clone(), errored_fn_ids);
        checker.visit_fn_item(&fn_.ast);
        errors.extend(checker.errors);
        all_errored_fn_ids = checker.errored_fn_ids;
    }
    analysis.errors.extend(errors);
}

#[derive(Debug)]
pub(crate) struct UsedFn {
    pub(crate) usage_span: Span,
    pub(crate) def_span: Span,
    pub(crate) id: FnId,
}

struct FnRecursionCheck<'a> {
    analysis: &'a Analysis,
    current_fn_id: FnId,
    called_fn_ids: Vec<UsedFn>,
    errored_fn_ids: FxHashSet<FnId>,
    errors: Vec<SemanticError>,
}

impl<'a> FnRecursionCheck<'a> {
    fn new(analysis: &'a Analysis, fn_id: FnId, errored_fn_ids: FxHashSet<FnId>) -> Self {
        Self {
            analysis,
            current_fn_id: fn_id,
            called_fn_ids: vec![],
            errored_fn_ids,
            errors: vec![],
        }
    }

    fn detect_error(&mut self) -> bool {
        if !self.is_last_call_recursive() {
            false
        } else if self.is_error_already_generated() {
            true
        } else {
            for call in &self.called_fn_ids {
                self.errored_fn_ids.insert(call.id.clone());
            }
            self.errored_fn_ids.insert(self.current_fn_id.clone());
            self.errors.push(errors::functions::recursion_found(
                &self.current_fn_id,
                &self.called_fn_ids,
            ));
            true
        }
    }

    fn is_last_call_recursive(&self) -> bool {
        self.called_fn_ids
            .last()
            .map_or(false, |last_call| last_call.id == self.current_fn_id)
    }

    fn is_error_already_generated(&self) -> bool {
        for call in &self.called_fn_ids {
            if self.errored_fn_ids.contains(&call.id) {
                return true;
            }
        }
        self.errored_fn_ids.contains(&self.current_fn_id)
    }
}

impl Visit for FnRecursionCheck<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = search::fn_(self.analysis, &node.name) {
            self.called_fn_ids.push(UsedFn {
                usage_span: node.span.clone(),
                def_span: fn_.ast.name.span.clone(),
                id: fn_.id.clone(),
            });
            if !self.detect_error() {
                self.visit_fn_item(&fn_.ast);
            }
            self.called_fn_ids.pop();
        }
    }
}

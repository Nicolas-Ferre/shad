use crate::{errors, Analysis, FnId};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use shad_parser::{AstFnCall, Visit};
use std::mem;

// TODO: fix error ordering

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_fn_ids = FxHashSet::default();
    for fn_ in analysis.fns.values() {
        let mut checker = FnRecursionCheck::new(
            analysis,
            FnId::new(&fn_.ast),
            mem::take(&mut errored_fn_ids),
        );
        checker.visit_fn_item(&fn_.ast);
        errors.extend(checker.errors);
        errored_fn_ids = checker.errored_fn_ids;
    }
    analysis.errors.extend(errors);
}

#[derive(Debug)]
pub(crate) struct CalledFn {
    pub(crate) call_span: Span,
    pub(crate) def_span: Span,
    pub(crate) id: FnId,
}

struct FnRecursionCheck<'a> {
    analysis: &'a Analysis,
    current_fn_id: FnId,
    called_fn_ids: Vec<CalledFn>,
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
        if let Some(id) = self.analysis.fn_id(&node.name) {
            let fn_ = &self.analysis.fns[&id].ast;
            self.called_fn_ids.push(CalledFn {
                call_span: node.span.clone(),
                def_span: fn_.name.span.clone(),
                id: id.clone(),
            });
            if !self.detect_error() {
                self.visit_fn_item(fn_);
            }
            self.called_fn_ids.pop();
        }
    }
}

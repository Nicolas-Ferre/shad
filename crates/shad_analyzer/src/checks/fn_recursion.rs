use crate::registration::functions::signature;
use crate::{errors, Analysis};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use shad_parser::{AstFnCall, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_fn_signatures = FxHashSet::default();
    for fn_ in analysis.fns.values() {
        let mut checker = FnRecursionCheck::new(
            analysis,
            signature(&fn_.ast),
            mem::take(&mut errored_fn_signatures),
        );
        checker.visit_fn_item(&fn_.ast);
        errors.extend(checker.errors);
        errored_fn_signatures = checker.errored_fn_signatures;
    }
    analysis.errors.extend(errors);
}

pub(crate) struct CalledFn {
    pub(crate) call_span: Span,
    pub(crate) fn_def_span: Span,
    pub(crate) signature: String,
}

struct FnRecursionCheck<'a> {
    analysis: &'a Analysis,
    current_fn_signature: String,
    called_fn_signatures: Vec<CalledFn>,
    errored_fn_signatures: FxHashSet<String>,
    errors: Vec<SemanticError>,
}

impl<'a> FnRecursionCheck<'a> {
    fn new(
        analysis: &'a Analysis,
        fn_signature: String,
        errored_fn_signatures: FxHashSet<String>,
    ) -> Self {
        Self {
            analysis,
            current_fn_signature: fn_signature,
            called_fn_signatures: vec![],
            errored_fn_signatures,
            errors: vec![],
        }
    }

    fn detect_error(&mut self) -> bool {
        if !self.is_last_call_recursive() {
            false
        } else if self.is_error_already_generated() {
            true
        } else {
            for call in &self.called_fn_signatures {
                self.errored_fn_signatures.insert(call.signature.clone());
            }
            self.errored_fn_signatures
                .insert(self.current_fn_signature.clone());
            self.errors.push(errors::functions::recursion_found(
                self.analysis,
                &self.current_fn_signature,
                &self.called_fn_signatures,
            ));
            true
        }
    }

    fn is_last_call_recursive(&self) -> bool {
        self.called_fn_signatures.last().map_or(false, |last_call| {
            last_call.signature == self.current_fn_signature
        })
    }

    fn is_error_already_generated(&self) -> bool {
        for call in &self.called_fn_signatures {
            if self.errored_fn_signatures.contains(&call.signature) {
                return true;
            }
        }
        self.errored_fn_signatures
            .contains(&self.current_fn_signature)
    }
}

impl Visit for FnRecursionCheck<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(signature) = self.analysis.fn_signature(&node.name) {
            let fn_ = &self.analysis.fns[&signature].ast;
            self.called_fn_signatures.push(CalledFn {
                call_span: node.span,
                fn_def_span: fn_.name.span,
                signature: signature.clone(),
            });
            if !self.detect_error() {
                self.visit_fn_item(fn_);
            }
            self.called_fn_signatures.pop();
        }
    }
}

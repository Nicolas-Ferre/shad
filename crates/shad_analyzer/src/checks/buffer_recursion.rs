use crate::{errors, search, Analysis, BufferId, IdentSource};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use shad_parser::{AstFnCall, AstIdent, Visit};
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_buffer_ids = FxHashSet::default();
    for buffer in analysis.buffers.values() {
        let mut checker = BufferRecursionCheck::new(
            analysis,
            BufferId::new(&buffer.ast),
            mem::take(&mut errored_buffer_ids),
        );
        checker.visit_expr(&buffer.ast.value);
        errors.extend(checker.errors);
        errored_buffer_ids = checker.errored_buffer_ids;
    }
    analysis.errors.extend(errors);
}

#[derive(Debug)]
pub(crate) struct UsedBuffer {
    pub(crate) usage_span: Span,
    pub(crate) def_span: Span,
    pub(crate) id: BufferId,
}

struct BufferRecursionCheck<'a> {
    analysis: &'a Analysis,
    current_buffer_id: BufferId,
    used_buffer_ids: Vec<UsedBuffer>,
    errored_buffer_ids: FxHashSet<BufferId>,
    errors: Vec<SemanticError>,
}

impl<'a> BufferRecursionCheck<'a> {
    fn new(
        analysis: &'a Analysis,
        buffer_id: BufferId,
        errored_buffer_ids: FxHashSet<BufferId>,
    ) -> Self {
        Self {
            analysis,
            current_buffer_id: buffer_id,
            used_buffer_ids: vec![],
            errored_buffer_ids,
            errors: vec![],
        }
    }

    fn detect_error(&mut self) -> bool {
        if !self.is_last_usage_recursive() {
            false
        } else if self.is_error_already_generated() {
            true
        } else {
            for call in &self.used_buffer_ids {
                self.errored_buffer_ids.insert(call.id.clone());
            }
            self.errored_buffer_ids
                .insert(self.current_buffer_id.clone());
            self.errors.push(errors::buffers::recursion_found(
                &self.current_buffer_id,
                &self.used_buffer_ids,
            ));
            true
        }
    }

    fn is_last_usage_recursive(&self) -> bool {
        self.used_buffer_ids
            .last()
            .map_or(false, |last_call| last_call.id == self.current_buffer_id)
    }

    fn is_error_already_generated(&self) -> bool {
        for call in &self.used_buffer_ids {
            if self.errored_buffer_ids.contains(&call.id) {
                return true;
            }
        }
        self.errored_buffer_ids.contains(&self.current_buffer_id)
    }
}

impl Visit for BufferRecursionCheck<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = search::fn_(self.analysis, &node.name) {
            self.visit_fn_item(&fn_.ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        if let IdentSource::Buffer(id) = &self.analysis.idents[&node.id].source {
            let buffer = &self.analysis.buffers[id].ast;
            self.used_buffer_ids.push(UsedBuffer {
                usage_span: node.span.clone(),
                def_span: buffer.name.span.clone(),
                id: id.clone(),
            });
            if !self.detect_error() {
                self.visit_expr(&buffer.value);
            }
            self.used_buffer_ids.pop();
        }
    }
}

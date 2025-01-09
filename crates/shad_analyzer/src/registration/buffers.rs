use crate::{errors, resolving, Analysis, TypeId};
use shad_parser::{AstBufferItem, AstItem};
use std::mem;

/// An analyzed buffer.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The buffer AST.
    pub ast: AstBufferItem,
    /// The unique identifier of the buffer.
    pub id: BufferId,
    /// The unique identifier of the buffer.
    pub type_id: Option<TypeId>,
}

/// The unique identifier of a buffer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId {
    /// The module in which the buffer is defined.
    pub module: String,
    /// The buffer name.
    pub name: String,
}

impl BufferId {
    pub(crate) fn new(buffer: &AstBufferItem) -> Self {
        Self {
            module: buffer.name.span.module.name.clone(),
            name: buffer.name.label.clone(),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_items(analysis);
    register_types(analysis);
}

fn register_items(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                let id = BufferId::new(buffer);
                let buffer_details = Buffer {
                    ast: buffer.clone(),
                    id: id.clone(),
                    type_id: None,
                };
                let existing_buffer = analysis.buffers.insert(id, buffer_details);
                if let Some(existing_buffer) = existing_buffer {
                    analysis
                        .errors
                        .push(errors::buffers::duplicated(buffer, &existing_buffer));
                }
            }
        }
    }
    analysis.asts = asts;
}

fn register_types(analysis: &mut Analysis) {
    let buffer_count = analysis.buffers.len();
    let mut typed_buffer_count = 0;
    let mut last_typed_buffer_count = 0;
    let buffer_ids: Vec<_> = analysis.buffers.keys().cloned().collect();
    while typed_buffer_count < buffer_count {
        for buffer_id in &buffer_ids {
            let buffer = &analysis.buffers[buffer_id];
            if buffer.type_id.is_none() {
                analysis
                    .buffers
                    .get_mut(buffer_id)
                    .expect("internal error: missing buffer")
                    .type_id = resolving::types::expr(analysis, &buffer.ast.value.clone());
            }
        }
        typed_buffer_count = count_typed_buffers(analysis);
        if typed_buffer_count == last_typed_buffer_count {
            break; // recursive buffer init
        }
        last_typed_buffer_count = typed_buffer_count;
    }
}

fn count_typed_buffers(analysis: &Analysis) -> usize {
    analysis
        .buffers
        .values()
        .filter(|buffer| buffer.type_id.is_some())
        .count()
}

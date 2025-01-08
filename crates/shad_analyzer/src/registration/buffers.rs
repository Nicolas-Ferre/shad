use crate::{errors, Analysis};
use shad_parser::{AstBufferItem, AstItem};
use std::mem;

/// An analyzed buffer.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The buffer AST.
    pub ast: AstBufferItem,
    /// The unique identifier of the buffer.
    pub id: BufferId,
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
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                let id = BufferId::new(buffer);
                let buffer_details = Buffer {
                    ast: buffer.clone(),
                    id: id.clone(),
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

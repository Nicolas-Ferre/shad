use crate::{errors, Analysis, BufferInitRunBlock};
use shad_parser::{AstBufferItem, AstItem};
use std::mem;

/// An analyzed buffer.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The buffer AST.
    pub ast: AstBufferItem,
}

pub(crate) fn register(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                let buffer_details = Buffer {
                    ast: buffer.clone(),
                };
                let existing_buffer = analysis
                    .buffers
                    .insert(BufferId::from_item(buffer), buffer_details);
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

/// The unique identifier of a buffer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId {
    /// The module in which the buffer is defined.
    pub module: String,
    /// The buffer name.
    pub name: String,
}

impl BufferId {
    pub(crate) fn from_run_block(block: &BufferInitRunBlock) -> Self {
        Self {
            module: block.module.clone(),
            name: block.buffer.clone(),
        }
    }

    pub(crate) fn from_item(buffer: &AstBufferItem) -> Self {
        Self {
            module: buffer.name.span.module.name.clone(),
            name: buffer.name.label.clone(),
        }
    }
}

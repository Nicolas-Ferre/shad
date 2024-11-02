use crate::{errors, Analysis};
use shad_parser::{AstBufferItem, AstItem};
use std::mem;

/// An analyzed buffer.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The buffer AST.
    pub ast: AstBufferItem,
}

pub(crate) fn register(analysis: &mut Analysis) {
    let items = mem::take(&mut analysis.ast.items);
    for item in &items {
        if let AstItem::Buffer(buffer) = item {
            let buffer_details = Buffer {
                ast: buffer.clone(),
            };
            let existing_buffer = analysis
                .buffers
                .insert(buffer.name.label.clone(), buffer_details);
            if let Some(existing_buffer) = existing_buffer {
                analysis.errors.push(errors::buffers::duplicated(
                    analysis,
                    buffer,
                    &existing_buffer,
                ));
            }
        }
    }
    analysis.ast.items = items;
}

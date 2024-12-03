use crate::checks::recursion::UsedItem;
use crate::{Buffer, BufferId};
use itertools::Itertools;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstBufferItem;
use std::iter;

pub(crate) fn duplicated(
    duplicated_buffer: &AstBufferItem,
    existing_buffer: &Buffer,
) -> SemanticError {
    SemanticError::new(
        format!(
            "buffer with name `{}` is defined multiple times",
            duplicated_buffer.name.label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_buffer.name.span.clone(),
                text: "duplicated buffer name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_buffer.ast.name.span.clone(),
                text: "buffer with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn recursion_found(
    current_buffer_id: &BufferId,
    buffer_stack: &[UsedItem<BufferId>],
) -> SemanticError {
    SemanticError::new(
        format!("buffer `{}` defined recursively", current_buffer_id.name),
        iter::once(LocatedMessage {
            level: ErrorLevel::Error,
            span: buffer_stack[buffer_stack.len() - 1].def_span.clone(),
            text: format!(
                "recursive buffer `{}` defined here",
                buffer_stack[buffer_stack.len() - 1].id.name
            ),
        })
        .chain(
            buffer_stack
                .iter()
                .circular_tuple_windows()
                .map(|(usage, next_usage)| LocatedMessage {
                    level: ErrorLevel::Info,
                    span: next_usage.usage_span.clone(),
                    text: format!(
                        "`{}` buffer used during `{}` buffer init",
                        next_usage.id.name, usage.id.name,
                    ),
                }),
        )
        .collect(),
    )
}

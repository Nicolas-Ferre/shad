use crate::Buffer;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstBufferItem;

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

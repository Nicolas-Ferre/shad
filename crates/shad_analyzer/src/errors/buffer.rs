use crate::{Asg, AsgBuffer};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstBufferItem;

pub(crate) fn duplicated(
    asg: &Asg,
    duplicated_buffer: &AstBufferItem,
    existing_buffer: &AsgBuffer,
) -> SemanticError {
    SemanticError::new(
        format!(
            "buffer with name `{}` is defined multiple times",
            duplicated_buffer.name.label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_buffer.name.span,
                text: "duplicated buffer name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_buffer.ast.name.span,
                text: "buffer with same name is defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

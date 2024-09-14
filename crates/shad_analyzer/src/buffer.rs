use crate::{Asg, AsgExpr};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstBufferItem, AstIdent};

/// An analyzed buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgBuffer {
    /// The buffer name in the initial Shad code.
    pub name: AstIdent,
    /// The unique buffer index.
    pub index: usize,
    /// The initial value of the buffer.
    pub expr: AsgExpr,
}

impl AsgBuffer {
    pub(crate) fn new(asg: &mut Asg, buffer: &AstBufferItem) -> Self {
        Self {
            name: buffer.name.clone(),
            index: asg.buffers.len(),
            expr: AsgExpr::new(asg, &buffer.value),
        }
    }
}

pub(crate) fn duplicated_error(
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
                span: existing_buffer.name.span,
                text: "buffer with same name is defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

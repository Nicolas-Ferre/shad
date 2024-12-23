use crate::TypeId;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstAssignment;

pub(crate) fn invalid_type(
    assignment: &AstAssignment,
    expected_type: &TypeId,
    expr_type: &TypeId,
) -> SemanticError {
    SemanticError::new(
        "invalid type in assignment",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: assignment.right.span().clone(),
                text: format!("expression of type `{}`", expr_type.name),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: assignment.left.span().clone(),
                text: format!("expected type `{}`", expected_type.name),
            },
        ],
    )
}

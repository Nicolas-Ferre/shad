use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstAssignment;

pub(crate) fn invalid_type(
    analysis: &Analysis,
    assignment: &AstAssignment,
    expected_type: &str,
    expr_type: &str,
) -> SemanticError {
    SemanticError::new(
        "invalid type in assignment",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: assignment.expr.span(),
                text: format!("expression of type `{expr_type}`"),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: assignment.value.span(),
                text: format!("expected type `{expected_type}`"),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

use crate::TypeId;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::AstExpr;

pub(crate) fn not_ref(expr: &AstExpr) -> SemanticError {
    SemanticError::new(
        "expression is not a reference",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: expr.span.clone(),
            text: "a valid reference is expected here".into(),
        }],
    )
}

pub(crate) fn invalid_type(
    expected_span: &Span,
    actual_span: &Span,
    expected_type: &TypeId,
    actual_type: &TypeId,
) -> SemanticError {
    SemanticError::new(
        "expression with invalid type",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: actual_span.clone(),
                text: format!("invalid type `{}`", actual_type.name),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: expected_span.clone(),
                text: format!("expected type `{}`", expected_type.name),
            },
        ],
    )
}

pub(crate) fn not_allowed_type(expr: &AstExpr, type_id: &TypeId) -> SemanticError {
    SemanticError::new(
        format!("expression of type `{}` is not allowed here", type_id.name),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: expr.span.clone(),
            text: "expression with invalid type".into(),
        }],
    )
}

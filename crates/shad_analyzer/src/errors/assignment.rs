use crate::{Asg, AsgAssignment, AsgType};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};

pub(crate) fn invalid_type(
    asg: &Asg,
    assignment: &AsgAssignment,
    assigned_type: &AsgType,
    expr_type: &AsgType,
) -> SemanticError {
    SemanticError::new(
        "invalid type in assignment",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: assignment.expr_span,
                text: format!("expression of type `{}`", expr_type.name.as_str()),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: assignment.assigned_span,
                text: format!("expected type `{}`", assigned_type.name.as_str()),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn not_ref_left_value(asg: &Asg, value_span: Span) -> SemanticError {
    SemanticError::new(
        "left value in assignment is not a reference",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: value_span,
            text: "this function doesn't return a reference".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

use crate::{Asg, AsgAssignment, AsgIdent, AsgType};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};

pub(crate) fn invalid_type(
    asg: &Asg,
    assignment: &AsgAssignment,
    assigned: &AsgIdent,
    assigned_type: &AsgType,
    expr_type: &AsgType,
) -> SemanticError {
    SemanticError::new(
        format!(
            "expression assigned to `{}` has invalid type",
            assigned.name()
        ),
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

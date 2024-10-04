use crate::{Asg, AsgLiteral};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};

pub(crate) fn invalid_integer(asg: &Asg, literal: &AsgLiteral, type_name: &str) -> SemanticError {
    SemanticError::new(
        format!("`{type_name}` literal out of range"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: literal.ast.span,
            text: format!("value is outside allowed range for `{type_name}` type"),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_f32(
    asg: &Asg,
    span: Span,
    digit_count: usize,
    int_part_limit: usize,
) -> SemanticError {
    SemanticError::new(
        "`f32` literal with too many digits in integer part",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span,
                text: format!("found {digit_count} digits"),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span,
                text: format!("maximum {int_part_limit} digits are expected"),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

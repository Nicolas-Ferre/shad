use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstLiteral;

pub(crate) fn invalid_integer(literal: &AstLiteral, type_name: &str) -> SemanticError {
    SemanticError::new(
        format!("`{type_name}` literal out of range"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: literal.span.clone(),
            text: format!("value is outside allowed range for `{type_name}` type"),
        }],
    )
}

pub(crate) fn too_many_f32_digits(
    literal: &AstLiteral,
    digit_count: usize,
    int_part_limit: usize,
) -> SemanticError {
    SemanticError::new(
        "`f32` literal with too many digits in integer part",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: literal.span.clone(),
                text: format!("found {digit_count} digits in integer part"),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: literal.span.clone(),
                text: format!("maximum {int_part_limit} digits are expected"),
            },
        ],
    )
}

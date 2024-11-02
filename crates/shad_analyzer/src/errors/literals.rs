use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstLiteral;

pub(crate) fn invalid_integer(
    analysis: &Analysis,
    literal: &AstLiteral,
    type_name: &str,
) -> SemanticError {
    SemanticError::new(
        format!("`{type_name}` literal out of range"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: literal.span,
            text: format!("value is outside allowed range for `{type_name}` type"),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn too_many_f32_digits(
    analysis: &Analysis,
    literal: &AstLiteral,
    digit_count: usize,
    int_part_limit: usize,
) -> SemanticError {
    SemanticError::new(
        "`f32` literal with too many digits in integer part",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: literal.span,
                text: format!("found {digit_count} digits in integer part"),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: literal.span,
                text: format!("maximum {int_part_limit} digits are expected"),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

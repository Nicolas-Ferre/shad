use crate::registration::constants::Constant;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstConstItem, AstExpr};

pub(crate) fn duplicated(
    duplicated_constant: &AstConstItem,
    existing_constant: &Constant,
) -> SemanticError {
    SemanticError::new(
        format!(
            "constant with name `{}` is defined multiple times",
            duplicated_constant.name.label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_constant.name.span.clone(),
                text: "duplicated constant name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_constant.ast.name.span.clone(),
                text: "constant with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn invalid_expr(expr: &AstExpr) -> SemanticError {
    SemanticError::new(
        "invalid expression in `const` context",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: expr.span.clone(),
            text: "not allowed in `const` context".into(),
        }],
    )
}

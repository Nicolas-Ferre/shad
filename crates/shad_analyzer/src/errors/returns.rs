use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnItem, AstReturn, AstStatement};

pub(crate) fn outside_fn(analysis: &Analysis, return_: &AstReturn) -> SemanticError {
    SemanticError::new(
        "`return` statement used outside function",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.span,
            text: "invalid statement".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn invalid_type(
    analysis: &Analysis,
    return_: &AstReturn,
    fn_: &AstFnItem,
    actual: &str,
    expected: &str,
) -> SemanticError {
    SemanticError::new(
        "invalid type for returned expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: return_.expr.span(),
                text: format!("expression of type `{actual}`"),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: fn_
                    .return_type
                    .as_ref()
                    .expect("internal error: no return type")
                    .name
                    .span,
                text: format!("expected type `{expected}`"),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn statement_after(
    analysis: &Analysis,
    return_: &AstStatement,
    next_statement: &AstStatement,
) -> SemanticError {
    SemanticError::new(
        "statement found after `return` statement",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: next_statement.span(),
                text: "this statement cannot be defined after a `return` statement".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: return_.span(),
                text: "`return` statement defined here".into(),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn no_return_type(analysis: &Analysis, return_: &AstReturn) -> SemanticError {
    SemanticError::new(
        "use of `return` in a function with no return type",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.span,
            text: "invalid statement".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn missing_return(
    analysis: &Analysis,
    fn_: &AstFnItem,
    signature: &str,
) -> SemanticError {
    SemanticError::new(
        format!("missing `return` statement in function `{signature}`"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_
                .return_type
                .as_ref()
                .expect("internal error: missing return type")
                .name
                .span,
            text: "the function should return a value".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

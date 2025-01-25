use crate::{FnId, TypeId};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnItem, AstReturn, AstStatement};

pub(crate) fn outside_fn(return_: &AstReturn) -> SemanticError {
    SemanticError::new(
        "`return` statement used outside function",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.span.clone(),
            text: "invalid statement".into(),
        }],
    )
}

pub(crate) fn invalid_type(
    return_: &AstReturn,
    fn_: &AstFnItem,
    actual: &TypeId,
    expected: &TypeId,
) -> SemanticError {
    SemanticError::new(
        "invalid type for returned expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: return_.expr.span.clone(),
                text: format!("expression of type `{}`", actual.name),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: fn_
                    .return_type
                    .as_ref()
                    .expect("internal error: no return type")
                    .type_
                    .span
                    .clone(),
                text: format!("expected type `{}`", expected.name),
            },
        ],
    )
}

pub(crate) fn statement_after(
    return_: &AstStatement,
    next_statement: &AstStatement,
) -> SemanticError {
    SemanticError::new(
        "statement found after `return` statement",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: next_statement.span().clone(),
                text: "this statement cannot be defined after a `return` statement".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: return_.span().clone(),
                text: "`return` statement defined here".into(),
            },
        ],
    )
}

pub(crate) fn no_return_type(return_: &AstReturn) -> SemanticError {
    SemanticError::new(
        "use of `return` in a function with no return type",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.span.clone(),
            text: "invalid statement".into(),
        }],
    )
}

pub(crate) fn missing_return(fn_: &AstFnItem, fn_id: &FnId) -> SemanticError {
    SemanticError::new(
        format!(
            "missing `return` statement in function `{}`",
            fn_id.signature()
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_
                .return_type
                .as_ref()
                .expect("internal error: missing return type")
                .type_
                .span
                .clone(),
            text: "the function should return a value".into(),
        }],
    )
}

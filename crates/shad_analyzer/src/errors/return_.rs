use crate::{Asg, AsgFn, AsgType};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstReturn, AstStatement};

pub(crate) fn outside_fn(asg: &Asg, statement: &AstReturn) -> SemanticError {
    SemanticError::new(
        "`return` statement used outside function",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: statement.span,
            text: "invalid statement".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_type(
    asg: &Asg,
    statement: &AstReturn,
    fn_: &AsgFn,
    actual: &AsgType,
    expected: &AsgType,
) -> SemanticError {
    SemanticError::new(
        "invalid type for returned expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: statement.expr.span(),
                text: format!("expression of type `{}`", actual.name.as_str()),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: fn_
                    .ast
                    .return_type
                    .as_ref()
                    .expect("internal error: no return type")
                    .span,
                text: format!("expected type `{}`", expected.name.as_str()),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn statement_after(
    asg: &Asg,
    statement: &AstStatement,
    return_span: Span,
) -> SemanticError {
    SemanticError::new(
        "statement found after `return` statement",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: statement.span(),
                text: "this statement cannot be defined after a `return` statement".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: return_span,
                text: "`return` statement defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn no_return_type(asg: &Asg, statement: &AstReturn) -> SemanticError {
    SemanticError::new(
        "use of `return` in a function with no return type",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: statement.span,
            text: "invalid statement".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

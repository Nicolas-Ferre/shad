use crate::errors::fn_::signature_str;
use crate::{Asg, AsgFn, AsgReturn, AsgType};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};

pub(crate) fn outside_fn(asg: &Asg, statement: &AsgReturn) -> SemanticError {
    SemanticError::new(
        "`return` statement used outside function",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: statement.ast.span,
            text: "invalid statement".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_type(
    asg: &Asg,
    return_: &AsgReturn,
    fn_: &AsgFn,
    actual: &AsgType,
    expected: &AsgType,
) -> SemanticError {
    SemanticError::new(
        "invalid type for returned expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: return_.ast.expr.span(),
                text: format!("expression of type `{}`", actual.name.as_str()),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: fn_
                    .ast
                    .return_type
                    .as_ref()
                    .expect("internal error: no return type")
                    .name
                    .span,
                text: format!("expected type `{}`", expected.name.as_str()),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn not_ref_expr(asg: &Asg, return_: &AsgReturn) -> SemanticError {
    SemanticError::new(
        "returned expression is not a reference",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.ast.expr.span(),
            text: "this expression is not a valid reference".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn statement_after(asg: &Asg, statement_span: Span, return_span: Span) -> SemanticError {
    SemanticError::new(
        "statement found after `return` statement",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: statement_span,
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

pub(crate) fn no_return_type(asg: &Asg, return_: &AsgReturn) -> SemanticError {
    SemanticError::new(
        "use of `return` in a function with no return type",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_.ast.span,
            text: "invalid statement".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn missing_return(asg: &Asg, fn_: &AsgFn) -> SemanticError {
    SemanticError::new(
        format!(
            "missing `return` statement in function `{}`",
            signature_str(&fn_.ast)
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_
                .ast
                .return_type
                .as_ref()
                .expect("internal error: missing return type")
                .name
                .span,
            text: "the function should return a value".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

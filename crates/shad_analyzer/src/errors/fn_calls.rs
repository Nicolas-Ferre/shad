use crate::FnId;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnCall, AstIdent};

pub(crate) fn no_return_type(fn_id: &FnId, fn_call: &AstFnCall) -> SemanticError {
    SemanticError::new(
        format!(
            "expected function with a return type, got function `{}`",
            fn_id.signature()
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_call.span.clone(),
            text: "this function cannot be called here".into(),
        }],
    )
}

pub(crate) fn invalid_param_name(arg_name: &AstIdent, param_name: &AstIdent) -> SemanticError {
    SemanticError::new(
        "invalid parameter name",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: arg_name.span.clone(),
                text: "invalid name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: param_name.span.clone(),
                text: "expected name".into(),
            },
        ],
    )
}

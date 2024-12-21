use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;

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

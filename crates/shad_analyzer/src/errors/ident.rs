use crate::Asg;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;

pub(crate) fn not_found(asg: &Asg, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` value", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined identifier".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

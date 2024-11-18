use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;

pub(crate) fn not_found(ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` type", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span.clone(),
            text: "undefined type".into(),
        }],
    )
}

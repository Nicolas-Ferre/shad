use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;

pub(crate) fn not_found(analysis: &Analysis, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` value", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined identifier".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

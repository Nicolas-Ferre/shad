use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;

pub(crate) fn not_found(analysis: &Analysis, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` type", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined type".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

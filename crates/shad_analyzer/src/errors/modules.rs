use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstImportItem;

pub(crate) fn not_found(import: &AstImportItem, module_name: &str) -> SemanticError {
    SemanticError::new(
        format!("could not find `{module_name}` module"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: import.span.clone(),
            text: "invalid import".into(),
        }],
    )
}

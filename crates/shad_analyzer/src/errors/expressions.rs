use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstExpr;

pub(crate) fn not_ref(expr: &AstExpr) -> SemanticError {
    SemanticError::new(
        "expression is not a reference",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: expr.span().clone(),
            text: "a valid reference is expected here".into(),
        }],
    )
}

use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstExpr;

pub(crate) fn not_ref(analysis: &Analysis, expr: &AstExpr) -> SemanticError {
    SemanticError::new(
        "expression is not a reference",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: expr.span(),
            text: "a valid reference is expected here".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

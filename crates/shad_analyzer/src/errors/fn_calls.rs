use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstExpr, AstFnCall};

pub(crate) fn not_allowed_buf_fn(
    analysis: &Analysis,
    call: &AstFnCall,
    signature: &str,
) -> SemanticError {
    SemanticError::new(
        format!("`buf` function `{signature}` called in invalid context"),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: call.span,
                text: "this function cannot be called here".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: call.span,
                text: "`buf` functions can only be called in `run` blocks and `buf fn` functions"
                    .into(),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn no_return_type(
    analysis: &Analysis,
    signature: &str,
    fn_call: &AstFnCall,
) -> SemanticError {
    SemanticError::new(
        format!("expected function with a return type, got function `{signature}`"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_call.span,
            text: "this function cannot be called here".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn unexpected_return_type(
    analysis: &Analysis,
    call: &AstFnCall,
    signature: &String,
) -> SemanticError {
    SemanticError::new(
        format!("function `{signature}` called as a statement while having a return type"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: call.span,
            text: "returned value needs to be stored in a variable".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn invalid_ref(analysis: &Analysis, expr: &AstExpr, ref_span: Span) -> SemanticError {
    SemanticError::new(
        "invalid reference expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: expr.span(),
                text: "not a reference".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: ref_span,
                text: "parameter is a reference".into(),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

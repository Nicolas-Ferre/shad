use crate::checks::fn_recursion::CalledFn;
use crate::Analysis;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnCall, AstFnItem, AstIdent};

pub(crate) fn duplicated(
    analysis: &Analysis,
    signature: &str,
    duplicated_fn: &AstFnItem,
    existing_fn: &AstFnItem,
) -> SemanticError {
    SemanticError::new(
        format!("function `{signature}` is defined multiple times"),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_fn.name.span,
                text: "duplicated function".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_fn.name.span,
                text: "function with same signature is defined here".into(),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn not_found(analysis: &Analysis, call: &AstFnCall, signature: &str) -> SemanticError {
    SemanticError::new(
        format!("could not find `{signature}` function"),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: call.span,
            text: "undefined function".into(),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn recursion_found(
    analysis: &Analysis,
    current_fn_signatures: &str,
    fn_stack: &[CalledFn],
) -> SemanticError {
    SemanticError::new(
        format!("function `{current_fn_signatures}` defined recursively"),
        fn_stack
            .iter()
            .flat_map(|call| {
                [
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span: call.call_span,
                        text: format!("`{}` function called here", call.signature),
                    },
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span: call.fn_def_span,
                        text: format!("`{}` function defined here", call.signature),
                    },
                ]
            })
            .collect(),
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn invalid_param_count(
    analysis: &Analysis,
    fn_: &AstFnItem,
    expected_count: usize,
) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` has an invalid number of parameters",
            fn_.name.label,
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_.name.span,
            text: format!(
                "found {} parameters, expected {expected_count}",
                fn_.params.len()
            ),
        }],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

pub(crate) fn duplicated_param(
    analysis: &Analysis,
    duplicated_param: &AstIdent,
    existing_param: &AstIdent,
) -> SemanticError {
    SemanticError::new(
        format!(
            "parameter `{}` is defined multiple times",
            &duplicated_param.label,
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_param.span,
                text: "duplicated parameter".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_param.span,
                text: "parameter with same name is defined here".into(),
            },
        ],
        &analysis.ast.code,
        &analysis.ast.path,
    )
}

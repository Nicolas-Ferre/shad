use crate::{Asg, AsgExpr, AsgFn, AsgFnCall, AsgFnSignature};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstFnItem, AstIdent};
use std::rc::Rc;

pub(crate) fn duplicated(
    asg: &Asg,
    duplicated_fn: &AstFnItem,
    existing_fn: &AsgFn,
) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` is defined multiple times",
            signature_str(duplicated_fn)
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_fn.name.span,
                text: "duplicated function".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_fn.ast.name.span,
                text: "function with same signature is defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn not_found(asg: &Asg, span: Span, signature: &AsgFnSignature) -> SemanticError {
    SemanticError::new(
        format!(
            "could not find `{}({})` function",
            signature.name,
            signature.param_types.join(", ")
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span,
            text: "undefined function".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn duplicated_param(
    asg: &Asg,
    duplicated_param_name: &AstIdent,
    existing_param_name: &AstIdent,
) -> SemanticError {
    SemanticError::new(
        format!(
            "parameter `{}` is defined multiple times",
            &duplicated_param_name.label,
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_param_name.span,
                text: "duplicated parameter".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_param_name.span,
                text: "parameter with same name is defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_param_count(asg: &Asg, fn_: &AsgFn, expected_count: usize) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` has an invalid number of parameters",
            fn_.ast.name.label,
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_.ast.name.span,
            text: format!(
                "found {} parameters, expected {expected_count}",
                fn_.params.len()
            ),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_ref(asg: &Asg, expr: &AsgExpr, ref_span: Span) -> SemanticError {
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
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn invalid_buf_fn_call(asg: &Asg, fn_call: &AsgFnCall) -> SemanticError {
    SemanticError::new(
        format!(
            "`buf` function `{}` called in invalid context",
            signature_str(&fn_call.fn_.ast)
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: fn_call.span,
                text: "this function cannot be called here".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: fn_call.span,
                text: "`buf` functions can only be called in `run` blocks and `buf fn` functions"
                    .into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn call_without_return_type_in_expr(asg: &Asg, fn_call: &AsgFnCall) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` in an expression while not having a return type",
            signature_str(&fn_call.fn_.ast)
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_call.span,
            text: "this function cannot be called here".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn call_with_return_type_in_statement(asg: &Asg, fn_call: &AsgFnCall) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` called as a statement while having a return type",
            signature_str(&fn_call.fn_.ast)
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_call.span,
            text: "returned value needs to be stored in a variable".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn recursion_found(
    asg: &Asg,
    current_fn: &Rc<AsgFn>,
    call_stack: &[(Span, Rc<AsgFn>)],
) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` defined recursively",
            signature_str(&current_fn.ast)
        ),
        call_stack
            .iter()
            .flat_map(|(span, fn_)| {
                [
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span: *span,
                        text: format!("`{}` function called here", signature_str(&fn_.ast)),
                    },
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span: fn_.ast.name.span,
                        text: format!("`{}` function defined here", signature_str(&fn_.ast)),
                    },
                ]
            })
            .collect(),
        &asg.code,
        &asg.path,
    )
}

pub(crate) fn signature_str(fn_: &AstFnItem) -> String {
    format!(
        "{}({})",
        &fn_.name.label,
        fn_.params
            .iter()
            .map(|param| param.type_.label.clone())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

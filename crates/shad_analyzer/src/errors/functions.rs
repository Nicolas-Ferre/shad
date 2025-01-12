use crate::checks::recursion::UsedItem;
use crate::{FnId, Function, GenericValue, TypeId};
use itertools::Itertools;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnCall, AstFnItem, AstIdent, AstReturnType};
use std::iter;

pub(crate) fn duplicated(
    id: &FnId,
    duplicated_fn: &AstFnItem,
    existing_fn: &AstFnItem,
) -> SemanticError {
    SemanticError::new(
        format!("function `{}` is defined multiple times", id.signature()),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_fn.name.span.clone(),
                text: "duplicated function".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_fn.name.span.clone(),
                text: "function with same signature is defined here".into(),
            },
        ],
    )
}

pub(crate) fn not_found(
    call: &AstFnCall,
    arg_types: &[TypeId],
    generic_args: &[GenericValue],
) -> SemanticError {
    SemanticError::new(
        format!(
            "could not find `{}{}({})` function",
            call.name.label,
            if generic_args.is_empty() { "" } else { "<...>" },
            arg_types.iter().map(|type_| &type_.name).join(", ")
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: call.span.clone(),
            text: "undefined function".into(),
        }],
    )
}

pub(crate) fn recursion_found(current_fn_id: &FnId, fn_stack: &[UsedItem<FnId>]) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` defined recursively",
            current_fn_id.signature()
        ),
        iter::once(LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_stack[fn_stack.len() - 1].def_span.clone(),
            text: format!(
                "recursive function `{}` defined here",
                fn_stack[fn_stack.len() - 1].id.signature()
            ),
        })
        .chain(
            fn_stack
                .iter()
                .circular_tuple_windows()
                .map(|(usage, next_usage)| LocatedMessage {
                    level: ErrorLevel::Info,
                    span: next_usage.usage_span.clone(),
                    text: format!(
                        "`{}` function called in `{}` function",
                        next_usage.id.signature(),
                        usage.id.signature(),
                    ),
                }),
        )
        .collect(),
    )
}

pub(crate) fn invalid_param_count(fn_: &AstFnItem, expected_count: usize) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` has an invalid number of parameters",
            fn_.name.label,
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_.name.span.clone(),
            text: format!(
                "found {} parameters, expected {expected_count}",
                fn_.params.len()
            ),
        }],
    )
}

pub(crate) fn duplicated_param(
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
                span: duplicated_param.span.clone(),
                text: "duplicated parameter".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_param.span.clone(),
                text: "parameter with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn not_found_const_fn(fn_: &Function) -> SemanticError {
    SemanticError::new(
        format!(
            "`const` function `{}` is not implemented in the compiler",
            fn_.id.signature()
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: fn_.ast.name.span.clone(),
            text: "undefined `const` function".into(),
        }],
    )
}

pub(crate) fn invalid_const_fn_return_type(
    fn_: &Function,
    return_type: &AstReturnType,
    expected_type_id: &TypeId,
    actual_type_id: &TypeId,
) -> SemanticError {
    SemanticError::new(
        format!(
            "`const` function `{}` has invalid return type `{}`, expected `{}`",
            fn_.id.signature(),
            actual_type_id.name,
            expected_type_id.name,
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: return_type.name.span.clone(),
            text: "invalid return type".into(),
        }],
    )
}

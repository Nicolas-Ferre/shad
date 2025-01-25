use crate::checks::recursion::UsedItem;
use crate::registration::constants::{Constant, ConstantId};
use itertools::Itertools;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstConstItem, AstFnCall, AstType};
use std::iter;

pub(crate) fn duplicated(
    duplicated_constant: &AstConstItem,
    existing_constant: &Constant,
) -> SemanticError {
    SemanticError::new(
        format!(
            "constant with name `{}` is defined multiple times",
            duplicated_constant.name.label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_constant.name.span.clone(),
                text: "duplicated constant name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_constant.ast.name.span.clone(),
                text: "constant with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn non_const_fn_call(call: &AstFnCall) -> SemanticError {
    SemanticError::new(
        "non-`const` function called in `const` context",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: call.span.clone(),
            text: "not allowed in `const` context".into(),
        }],
    )
}

pub(crate) fn recursion_found(
    current_constant_id: &ConstantId,
    constant_stack: &[UsedItem<ConstantId>],
) -> SemanticError {
    SemanticError::new(
        format!(
            "constant `{}` defined recursively",
            current_constant_id.name
        ),
        iter::once(LocatedMessage {
            level: ErrorLevel::Error,
            span: constant_stack[constant_stack.len() - 1].def_span.clone(),
            text: format!(
                "recursive constant `{}` defined here",
                constant_stack[constant_stack.len() - 1].id.name
            ),
        })
        .chain(
            constant_stack
                .iter()
                .circular_tuple_windows()
                .map(|(usage, next_usage)| LocatedMessage {
                    level: ErrorLevel::Info,
                    span: next_usage.usage_span.clone(),
                    text: format!(
                        "`{}` constant used during `{}` constant init",
                        next_usage.id.name, usage.id.name,
                    ),
                }),
        )
        .collect(),
    )
}

pub(crate) fn unsupported_type(type_: &AstType) -> SemanticError {
    SemanticError::new(
        format!(
            "unsupported type `{}` in `const` context, expected `u32`, `i32`, `f32` or `bool`",
            type_.name.label
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: type_.span.clone(),
            text: "unsupported type".into(),
        }],
    )
}

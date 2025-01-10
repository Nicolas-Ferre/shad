use crate::checks::recursion::UsedItem;
use crate::TypeId;
use itertools::Itertools;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstGpuQualifier, AstIdent, AstStructItem};
use std::iter;

pub(crate) fn duplicated(
    id: &TypeId,
    duplicated_type: &AstStructItem,
    existing_type: &AstStructItem,
) -> SemanticError {
    SemanticError::new(
        format!("type `{}` is defined multiple times", id.name),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_type.name.span.clone(),
                text: "duplicated type".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_type.name.span.clone(),
                text: "type with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn not_found(ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` type", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span.clone(),
            text: "undefined type".into(),
        }],
    )
}

pub(crate) fn recursion_found(
    current_type_id: &TypeId,
    type_stack: &[UsedItem<TypeId>],
) -> SemanticError {
    SemanticError::new(
        format!("type `{}` defined recursively", current_type_id.name),
        iter::once(LocatedMessage {
            level: ErrorLevel::Error,
            span: type_stack[type_stack.len() - 1].def_span.clone(),
            text: format!(
                "recursive type `{}` defined here",
                type_stack[type_stack.len() - 1].id.name
            ),
        })
        .chain(
            type_stack
                .iter()
                .circular_tuple_windows()
                .map(|(usage, next_usage)| LocatedMessage {
                    level: ErrorLevel::Info,
                    span: next_usage.usage_span.clone(),
                    text: format!(
                        "`{}` type used in `{}` type",
                        next_usage.id.name, usage.id.name,
                    ),
                }),
        )
        .collect(),
    )
}

pub(crate) fn field_not_found(field: &AstIdent, type_id: &TypeId) -> SemanticError {
    SemanticError::new(
        format!(
            "could not find `{}` field in `{}` type",
            field.label, type_id.name
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: field.span.clone(),
            text: "undefined field".into(),
        }],
    )
}

pub(crate) fn no_field(ast: &AstStructItem) -> SemanticError {
    SemanticError::new(
        format!("struct `{}` without field", ast.name.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ast.name.span.clone(),
            text: "invalid struct".into(),
        }],
    )
}

pub(crate) fn missing_layout(ast: &AstStructItem, gpu: &AstGpuQualifier) -> SemanticError {
    SemanticError::new(
        format!(
            "missing layout definition for the `gpu` struct `{}`",
            ast.name.label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: ast.name.span.clone(),
                text: "invalid struct definition".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: gpu.span.clone(),
                text: "`gpu` structs should have a `layout`".into(),
            },
        ],
    )
}

pub(crate) fn invalid_gpu_array_args(gpu: &AstGpuQualifier) -> SemanticError {
    SemanticError::new(
        "invalid `gpu` array generic params",
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: gpu.span.clone(),
            text: "`gpu` array should have two generic arguments (a type and a non-zero positive 32-bit integer)".into(),
        }],
    )
}

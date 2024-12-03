use crate::checks::type_recursion::UsedType;
use crate::TypeId;
use itertools::Itertools;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstIdent, AstStructItem};
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

pub(crate) fn recursion_found(current_type_id: &TypeId, type_stack: &[UsedType]) -> SemanticError {
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

use crate::TypeId;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstIdent, AstStructItem};

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

pub(crate) fn invalid_field_type(struct_: &AstStructItem, field_type: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!(
            "not allowed type `{}` in struct `{}`",
            field_type.label, struct_.name.label
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: field_type.span.clone(),
            text: "only `u32`, `i32`, `f32` and `bool` types are allowed here".into(),
        }],
    )
}

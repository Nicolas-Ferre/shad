use crate::{errors, Analysis};
use shad_parser::AstIdent;

/// The 32-bit floating point type name.
pub const F32_TYPE: &str = "f32";
/// The 32-bit unsigned integer type name.
pub const U32_TYPE: &str = "u32";
/// The 32-bit signed integer type name.
pub const I32_TYPE: &str = "i32";
/// The boolean type name.
pub const BOOL_TYPE: &str = "bool";

/// An analyzed type.
#[derive(Debug, Clone)]
pub struct Type {
    /// The type name.
    pub name: String,
    /// The type name when used for a buffer.
    pub buffer_name: String,
    /// The type size in bytes.
    pub size: usize,
}

pub(crate) fn register(analysis: &mut Analysis) {
    analysis.types = [
        Type {
            name: F32_TYPE.into(),
            buffer_name: F32_TYPE.into(),
            size: 4,
        },
        Type {
            name: U32_TYPE.into(),
            buffer_name: U32_TYPE.into(),
            size: 4,
        },
        Type {
            name: I32_TYPE.into(),
            buffer_name: I32_TYPE.into(),
            size: 4,
        },
        Type {
            name: BOOL_TYPE.into(),
            buffer_name: U32_TYPE.into(),
            size: 4,
        },
    ]
    .into_iter()
    .map(|type_| (type_.name.clone(), type_))
    .collect();
}

pub(crate) fn name(analysis: &mut Analysis, ident: &AstIdent) -> Option<String> {
    exists(analysis, ident).then(|| ident.label.clone())
}

fn exists(analysis: &mut Analysis, ident: &AstIdent) -> bool {
    if analysis.types.contains_key(&ident.label) {
        true
    } else {
        analysis.errors.push(errors::types::not_found(ident));
        false
    }
}

use crate::{errors, Analysis};
use shad_parser::{AstIdent, AstItem, AstStructItem};
use std::mem;

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
    /// The type AST.
    pub ast: Option<AstStructItem>,
}

/// The unique identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeId {
    /// The module in which the type is defined.
    pub module: Option<String>,
    /// The type name.
    pub name: String,
}

impl TypeId {
    pub(crate) fn from_builtin(name: &str) -> Self {
        Self {
            module: None,
            name: name.into(),
        }
    }

    pub(crate) fn from_struct(fn_: &AstStructItem) -> Self {
        Self {
            module: Some(fn_.name.span.module.name.clone()),
            name: fn_.name.label.clone(),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_builtin(analysis);
    register_structs(analysis);
}

fn register_builtin(analysis: &mut Analysis) {
    analysis.types.extend(
        [
            Type {
                name: F32_TYPE.into(),
                buffer_name: F32_TYPE.into(),
                size: 4,
                ast: None,
            },
            Type {
                name: U32_TYPE.into(),
                buffer_name: U32_TYPE.into(),
                size: 4,
                ast: None,
            },
            Type {
                name: I32_TYPE.into(),
                buffer_name: I32_TYPE.into(),
                size: 4,
                ast: None,
            },
            Type {
                name: BOOL_TYPE.into(),
                buffer_name: U32_TYPE.into(),
                size: 4,
                ast: None,
            },
        ]
        .into_iter()
        .map(|type_| (TypeId::from_builtin(&type_.name), type_)),
    );
}

fn register_structs(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for items in &ast.items {
            if let AstItem::Struct(struct_) = items {
                let id = TypeId::from_struct(struct_);
                let existing_struct = analysis.types.insert(
                    id.clone(),
                    Type {
                        name: struct_.name.label.clone(),
                        buffer_name: struct_.name.label.clone(),
                        size: 0,
                        ast: Some(struct_.clone()),
                    },
                );
                if let Some(existing_struct) = existing_struct {
                    analysis.errors.push(errors::types::duplicated(
                        &id,
                        struct_,
                        existing_struct
                            .ast
                            .as_ref()
                            .expect("internal error: missing type AST"),
                    ));
                }
            }
        }
    }
    analysis.asts = asts;
}

pub(crate) fn find(analysis: &Analysis, module: &str, ident: &AstIdent) -> Option<TypeId> {
    let type_id = TypeId {
        module: Some(module.into()),
        name: ident.label.clone(),
    };
    if analysis.types.contains_key(&type_id) {
        return Some(type_id);
    }
    let builtin_type_id = TypeId::from_builtin(&ident.label);
    if analysis.types.contains_key(&builtin_type_id) {
        Some(builtin_type_id)
    } else {
        None
    }
}

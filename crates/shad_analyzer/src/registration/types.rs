use crate::{errors, Analysis};
use shad_error::SemanticError;
use shad_parser::{AstIdent, AstItem, AstStructField, AstStructItem};
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
    /// The fields of the type when this is a struct.
    pub fields: Vec<StructField>,
}

/// The unique identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeId {
    /// The module in which the type is defined.
    pub module: Option<String>,
    /// The type name.
    pub name: String,
}

/// The unique identifier of a function.
#[derive(Debug, Clone)]
pub struct StructField {
    /// The field name.
    pub name: AstIdent,
    /// The field type ID.
    pub type_id: Option<TypeId>,
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
    register_struct_details(analysis);
}

fn register_builtin(analysis: &mut Analysis) {
    analysis.types.extend(
        [
            Type {
                name: F32_TYPE.into(),
                buffer_name: F32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                name: U32_TYPE.into(),
                buffer_name: U32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                name: I32_TYPE.into(),
                buffer_name: I32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                name: BOOL_TYPE.into(),
                buffer_name: U32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
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
                let type_ = Type {
                    name: struct_.name.label.clone(),
                    buffer_name: struct_.name.label.clone(),
                    size: 0, // defined once all structs have been detected
                    ast: Some(struct_.clone()),
                    fields: vec![], // defined once all structs have been detected
                };
                if let Some(existing_struct) = analysis.types.insert(id.clone(), type_) {
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

fn register_struct_details(analysis: &mut Analysis) {
    analysis.types = analysis
        .types
        .clone()
        .into_iter()
        .map(|(type_id, mut type_)| {
            if let Some(ast) = &type_.ast {
                type_.fields = ast
                    .fields
                    .iter()
                    .map(|field| analyze_field(analysis, field))
                    .collect();
                type_.size = type_
                    .fields
                    .iter()
                    .flat_map(|field| &field.type_id)
                    .map(|type_id| analysis.types[type_id].size)
                    .sum();
            }
            (type_id, type_)
        })
        .collect();
}

fn analyze_field(analysis: &mut Analysis, field: &AstStructField) -> StructField {
    StructField {
        name: field.name.clone(),
        type_id: find_or_add_error(analysis, &field.type_),
    }
}

pub(crate) fn find_or_add_error(analysis: &mut Analysis, ident: &AstIdent) -> Option<TypeId> {
    match find(analysis, ident) {
        Ok(type_id) => Some(type_id),
        Err(error) => {
            analysis.errors.push(error);
            None
        }
    }
}

pub(crate) fn find(analysis: &Analysis, ident: &AstIdent) -> Result<TypeId, SemanticError> {
    let type_id = TypeId {
        module: Some(ident.span.module.name.clone()),
        name: ident.label.clone(),
    };
    if analysis.types.contains_key(&type_id) {
        return Ok(type_id);
    }
    let builtin_type_id = TypeId::from_builtin(&ident.label);
    if analysis.types.contains_key(&builtin_type_id) {
        Ok(builtin_type_id)
    } else {
        Err(errors::types::not_found(ident))
    }
}

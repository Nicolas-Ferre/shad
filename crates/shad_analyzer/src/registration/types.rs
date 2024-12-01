use crate::{errors, resolver, Analysis};
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
    /// The unique ID of the type.
    pub id: TypeId,
    /// The type name.
    pub name: String,
    /// The type size in bytes.
    pub size: usize,
    /// The type AST.
    pub ast: Option<AstStructItem>,
    /// The fields of the type when this is a struct.
    pub fields: Vec<StructField>,
}

/// The unique identifier of a type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeId {
    /// The module in which the type is defined.
    pub module: Option<String>,
    /// The type name.
    pub name: String,
}

/// An analyzed struct field.
#[derive(Debug, Clone)]
pub struct StructField {
    /// The field name.
    pub name: AstIdent,
    /// The field type ID.
    pub type_id: Option<TypeId>,
}

impl TypeId {
    /// Creates the type ID of a builtin type.
    pub fn from_builtin(name: &str) -> Self {
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
                id: TypeId::from_builtin(F32_TYPE),
                name: F32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                id: TypeId::from_builtin(U32_TYPE),
                name: U32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                id: TypeId::from_builtin(I32_TYPE),
                name: I32_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
            Type {
                id: TypeId::from_builtin(BOOL_TYPE),
                name: BOOL_TYPE.into(),
                size: 4,
                ast: None,
                fields: vec![],
            },
        ]
        .into_iter()
        .map(|type_| (type_.id.clone(), type_)),
    );
}

fn register_structs(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for items in &ast.items {
            if let AstItem::Struct(struct_) = items {
                let id = TypeId::from_struct(struct_);
                let type_ = Type {
                    id: id.clone(),
                    name: struct_.name.label.clone(),
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
        type_id: resolver::type_or_add_error(analysis, &field.type_),
    }
}

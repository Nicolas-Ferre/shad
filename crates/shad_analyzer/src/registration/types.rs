use crate::registration::generics;
use crate::registration::generics::GenericParam;
use crate::{errors, resolving, Analysis};
use shad_error::SemanticError;
use shad_parser::{AstGpuGenericParam, AstGpuQualifier, AstIdent, AstItem, AstStructItem};
use std::mem;
use std::num::NonZeroU32;
use std::str::FromStr;

/// The no return type name.
pub const NO_RETURN_TYPE: &str = "<no return>";
/// The 32-bit floating point type name.
pub const F32_TYPE: &str = "f32";
/// The 32-bit unsigned integer type name.
pub const U32_TYPE: &str = "u32";
/// The 32-bit signed integer type name.
pub const I32_TYPE: &str = "i32";
/// The boolean type name.
pub const BOOL_TYPE: &str = "bool";
pub(crate) const WGSL_ARRAY_TYPE: &str = "array";

/// An analyzed type.
#[derive(Debug, Clone)]
pub struct Type {
    /// The unique ID of the type.
    pub id: TypeId,
    /// The type name.
    pub name: String,
    /// The type size in bytes.
    pub size: u32,
    /// The type alignment in bytes.
    pub alignment: u32,
    /// The type AST.
    pub ast: Option<AstStructItem>,
    /// The fields of the type when this is a struct.
    pub fields: Vec<StructField>,
    /// The analyzed generic parameters of the type.
    pub generics: Vec<GenericParam>,
    /// The analyzed generic arguments when the `gpu` type is an array.
    pub array_params: Option<(TypeId, u32)>,
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
    /// Whether the item is public.
    pub is_pub: bool,
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
    register_array_params(analysis);
    register_struct_details(analysis);
}

fn register_builtin(analysis: &mut Analysis) {
    analysis.types.extend(
        [
            Type {
                id: TypeId::from_builtin(NO_RETURN_TYPE),
                name: NO_RETURN_TYPE.into(),
                size: 0,
                alignment: 0,
                ast: None,
                fields: vec![],
                generics: vec![],
                array_params: None,
            },
            Type {
                id: TypeId::from_builtin(F32_TYPE),
                name: F32_TYPE.into(),
                size: 4,
                alignment: 4,
                ast: None,
                fields: vec![],
                generics: vec![],
                array_params: None,
            },
            Type {
                id: TypeId::from_builtin(U32_TYPE),
                name: U32_TYPE.into(),
                size: 4,
                alignment: 4,
                ast: None,
                fields: vec![],
                generics: vec![],
                array_params: None,
            },
            Type {
                id: TypeId::from_builtin(I32_TYPE),
                name: I32_TYPE.into(),
                size: 4,
                alignment: 4,
                ast: None,
                fields: vec![],
                generics: vec![],
                array_params: None,
            },
            Type {
                id: TypeId::from_builtin(BOOL_TYPE),
                name: BOOL_TYPE.into(),
                size: 4,
                alignment: 4,
                ast: None,
                fields: vec![],
                generics: vec![],
                array_params: None,
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
                    size: 0,      // defined once all structs have been detected
                    alignment: 0, // defined once all structs have been detected
                    ast: Some(struct_.clone()),
                    fields: vec![],     // defined once all structs have been detected
                    generics: vec![],   // defined once all structs have been detected
                    array_params: None, // defined once all structs have been detected
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

fn register_array_params(analysis: &mut Analysis) {
    analysis.types = analysis
        .types
        .clone()
        .into_iter()
        .map(|(type_id, mut type_)| {
            parse_array_params(analysis, &mut type_);
            (type_id, type_)
        })
        .collect();
}

fn register_struct_details(analysis: &mut Analysis) {
    let mut sized_count = 0;
    let mut last_sized_count = 0;
    while sized_count < analysis.types.len() {
        analysis.types = analysis
            .types
            .clone()
            .into_iter()
            .map(|(type_id, mut type_)| {
                calculate_type_details(analysis, &mut type_);
                (type_id, type_)
            })
            .collect();
        sized_count = analysis
            .types
            .values()
            .filter(|type_| type_.size > 0)
            .count();
        if sized_count == last_sized_count {
            break; // recursive struct definition
        }
        last_sized_count = sized_count;
    }
}

fn parse_array_params(analysis: &mut Analysis, type_: &mut Type) {
    if let Some(ast) = &type_.ast {
        if let (Some(gpu), None) = (&ast.gpu_qualifier, &ast.layout) {
            if let Some(name) = &gpu.name {
                if name.root.label == WGSL_ARRAY_TYPE {
                    match parse_array_generic_args(analysis, gpu, &name.generics) {
                        Ok(params) => type_.array_params = Some(params),
                        Err(Some(err)) => analysis.errors.push(err),
                        Err(None) => (),
                    }
                }
            }
        }
    }
}

fn parse_array_generic_args(
    analysis: &Analysis,
    gpu: &AstGpuQualifier,
    generics: &[AstGpuGenericParam],
) -> Result<(TypeId, u32), Option<SemanticError>> {
    if let (Some(AstGpuGenericParam::Ident(item_type)), Some(AstGpuGenericParam::Literal(length))) =
        (generics.first(), generics.get(1))
    {
        let module = &gpu.span.module.name;
        let item_type = resolving::items::type_id(analysis, module, item_type).map_err(|_| None)?;
        let length = NonZeroU32::from_str(&length.value.replace('_', ""))
            .map_err(|_| errors::types::invalid_gpu_array_args(gpu))?;
        Ok((item_type, length.into()))
    } else {
        Err(Some(errors::types::invalid_gpu_array_args(gpu)))
    }
}

fn calculate_type_details(analysis: &mut Analysis, type_: &mut Type) {
    if let Some(ast) = &type_.ast {
        if type_.fields.is_empty() {
            type_.fields = analyze_fields(analysis, ast);
        }
        if type_.generics.is_empty() && !ast.generics.params.is_empty() {
            let module = &ast.name.span.module.name;
            type_.generics = generics::register_for_item(analysis, &ast.generics, module);
        }
        let are_fields_registered = type_
            .fields
            .iter()
            .flat_map(|field| &field.type_id)
            .all(|type_id| analysis.types[type_id].size > 0);
        if are_fields_registered {
            if let Some((size, alignment)) = calculate_layout(analysis, type_, ast) {
                type_.size = size;
                type_.alignment = alignment;
            }
        }
    }
}

fn analyze_fields(analysis: &mut Analysis, ast: &AstStructItem) -> Vec<StructField> {
    ast.fields
        .iter()
        .map(|field| {
            let module = &ast.name.span.module.name;
            StructField {
                name: field.name.clone(),
                type_id: resolving::items::type_id_or_add_error(analysis, module, &field.type_),
                is_pub: field.is_pub,
            }
        })
        .collect()
}

fn calculate_layout(analysis: &Analysis, type_: &Type, ast: &AstStructItem) -> Option<(u32, u32)> {
    if let Some((item_type_id, length)) = &type_.array_params {
        let item_type = &analysis.types[item_type_id];
        if item_type.size == 0 {
            return None; // item type layout not yet calculated
        }
        Some((
            length * round_up(item_type.alignment, item_type.size),
            item_type.alignment,
        ))
    } else if let Some(layout) = &ast.layout {
        Some((layout.size.into(), layout.alignment.into()))
    } else if !type_.fields.is_empty() {
        let alignment = type_
            .fields
            .iter()
            .flat_map(|field| &field.type_id)
            .map(|type_id| analysis.types[type_id].alignment)
            .max()
            .unwrap_or(0);
        let last_field_type_id = type_.fields[type_.fields.len() - 1].type_id.as_ref()?;
        let last_field_size = analysis.types[last_field_type_id].size;
        Some((
            round_up(
                alignment,
                struct_offset(analysis, &type_.fields)? + last_field_size,
            ),
            alignment,
        ))
    } else {
        None
    }
}

fn struct_offset(analysis: &Analysis, fields: &[StructField]) -> Option<u32> {
    Some(if fields.len() == 1 {
        0
    } else {
        let last_field_type_id = fields[fields.len() - 1].type_id.as_ref()?;
        let before_last_field_type_id = fields[fields.len() - 2].type_id.as_ref()?;
        let last_field_alignment = analysis.types[last_field_type_id].alignment;
        let before_last_field_size = analysis.types[before_last_field_type_id].size;
        round_up(
            last_field_alignment,
            struct_offset(analysis, &fields[..fields.len() - 1])? + before_last_field_size,
        )
    })
}

fn round_up(k: u32, n: u32) -> u32 {
    n.div_ceil(k) * k
}

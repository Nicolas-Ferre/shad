use crate::{resolving, Analysis, TypeId};
use shad_parser::{AstGpuGenericParam, AstGpuQualifier, AstIdent, AstItemGenerics};

/// An analyzed generic parameter.
#[derive(Debug, Clone)]
pub enum GenericParam {
    /// A type.
    Type(TypeGenericParam),
    /// A constant.
    Constant(ConstantGenericParam),
}

impl GenericParam {
    pub(crate) fn name(&self) -> &AstIdent {
        match self {
            Self::Type(param) => &param.name,
            Self::Constant(param) => &param.name,
        }
    }
}

/// An analyzed type generic parameter.
#[derive(Debug, Clone)]
pub struct TypeGenericParam {
    /// The parameter name.
    pub name: AstIdent,
}

/// An analyzed constant generic parameter.
#[derive(Debug, Clone)]
pub struct ConstantGenericParam {
    /// The parameter name.
    pub name: AstIdent,
    /// The parameter type name.
    pub type_name: AstIdent,
    /// The parameter type identifier.
    pub type_id: Option<TypeId>,
}

pub(crate) fn register_for_item(
    analysis: &mut Analysis,
    generics: &AstItemGenerics,
) -> Vec<GenericParam> {
    generics
        .params
        .iter()
        .map(|param| {
            let name = param.name.clone();
            if let Some(type_) = &param.type_ {
                GenericParam::Constant(ConstantGenericParam {
                    name,
                    type_name: type_.clone(),
                    type_id: resolving::items::type_id_or_add_error(analysis, type_),
                })
            } else {
                GenericParam::Type(TypeGenericParam { name })
            }
        })
        .collect()
}

pub(crate) fn register_gpu_qualifier(analysis: &Analysis, qualifier: &mut Option<AstGpuQualifier>) {
    if let Some(qualifier) = qualifier {
        if let Some(name) = &mut qualifier.name {
            for param in &mut name.generics {
                if let AstGpuGenericParam::Ident(ident) = param {
                    if let Ok(type_id) = resolving::items::type_id(analysis, ident) {
                        ident.label = type_id;
                    }
                }
            }
        }
    }
}

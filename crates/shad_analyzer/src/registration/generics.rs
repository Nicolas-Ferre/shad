use crate::{resolving, Analysis, TypeId};
use shad_parser::{AstIdent, AstItemGenerics, AstType};

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
    /// The parameter type.
    pub type_: AstType,
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
                    type_: type_.clone(),
                    type_id: resolving::items::type_id_or_add_error(analysis, type_),
                })
            } else {
                GenericParam::Type(TypeGenericParam { name })
            }
        })
        .collect()
}

use crate::{errors, Analysis, Function, IdentSource, TypeId};
use shad_error::SemanticError;
use shad_parser::AstIdent;

pub(crate) fn type_or_add_error(analysis: &mut Analysis, type_: &AstIdent) -> Option<TypeId> {
    match self::type_(analysis, type_) {
        Ok(type_id) => Some(type_id),
        Err(error) => {
            analysis.errors.push(error);
            None
        }
    }
}

pub(crate) fn type_(analysis: &Analysis, type_: &AstIdent) -> Result<TypeId, SemanticError> {
    let type_id = TypeId {
        module: Some(type_.span.module.name.clone()),
        name: type_.label.clone(),
    };
    if analysis.types.contains_key(&type_id) {
        return Ok(type_id);
    }
    let builtin_type_id = TypeId::from_builtin(&type_.label);
    if analysis.types.contains_key(&builtin_type_id) {
        Ok(builtin_type_id)
    } else {
        Err(errors::types::not_found(type_))
    }
}

pub(crate) fn fn_<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Function> {
    analysis
        .idents
        .get(&name.id)
        .map(|ident| match &ident.source {
            IdentSource::Fn(id) => id.clone(),
            IdentSource::Buffer(_) | IdentSource::Var(_) => {
                unreachable!("internal error: retrieve non-function ID")
            }
        })
        .map(|fn_id| &analysis.fns[&fn_id])
}

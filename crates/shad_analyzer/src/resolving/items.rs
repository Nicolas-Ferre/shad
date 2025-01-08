use crate::registration::constants::{Constant, ConstantId};
use crate::{errors, Analysis, Buffer, BufferId, FnId, Function, IdentSource, TypeId};
use shad_error::SemanticError;
use shad_parser::{AstFnCall, AstIdent};
use std::iter;

pub(crate) fn type_id_or_add_error(analysis: &mut Analysis, name: &AstIdent) -> Option<TypeId> {
    match type_id(analysis, name) {
        Ok(type_id) => Some(type_id),
        Err(error) => {
            analysis.errors.push(error);
            None
        }
    }
}

pub(crate) fn type_id(analysis: &Analysis, name: &AstIdent) -> Result<TypeId, SemanticError> {
    let module = &name.span.module.name;
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .map(Some)
        .chain(iter::once(None))
        .filter_map(|module| {
            let id = TypeId {
                module: module.cloned(),
                name: name.label.clone(),
            };
            analysis.types.get(&id).map(|type_| (id, type_))
        })
        .find(|(type_id, type_)| {
            type_.ast.as_ref().map_or(true, |ast| ast.is_pub)
                || type_id.module.as_deref() == Some(module)
        })
        .map(|(type_id, _)| type_id)
        .ok_or_else(|| errors::types::not_found(name))
}

pub(crate) fn buffer<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Buffer> {
    let module = &name.span.module.name;
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .filter_map(|module| {
            let id = BufferId {
                module: module.clone(),
                name: name.label.clone(),
            };
            analysis.buffers.get(&id)
        })
        .find(|buffer| buffer.ast.is_pub || &buffer.id.module == module)
}

pub(crate) fn constant<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Constant> {
    let module = &name.span.module.name;
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .filter_map(|module| {
            let id = ConstantId {
                module: module.clone(),
                name: name.label.clone(),
            };
            analysis.constants.get(&id)
        })
        .find(|constant| constant.ast.is_pub || &constant.id.module == module)
}

pub(crate) fn fn_<'a>(
    analysis: &'a Analysis,
    call: &AstFnCall,
    arg_types: &[TypeId],
) -> Option<&'a Function> {
    let module = &call.name.span.module.name;
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .filter_map(|module| {
            let id = FnId {
                module: module.clone(),
                name: call.name.label.clone(),
                param_types: arg_types.iter().map(|type_| Some(type_.clone())).collect(),
                param_count: arg_types.len(),
                is_generic: false,
            };
            analysis.fns.get(&id)
        })
        .find(|fn_| fn_.ast.is_pub || &fn_.id.module == module)
}

pub(crate) fn registered_fn<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Function> {
    analysis
        .idents
        .get(&name.id)
        .map(|ident| match &ident.source {
            IdentSource::Fn(id) => id.clone(),
            IdentSource::Constant(_)
            | IdentSource::Buffer(_)
            | IdentSource::Var(_)
            | IdentSource::Field
            | IdentSource::GenericType => {
                unreachable!("internal error: retrieve non-function ID")
            }
        })
        .map(|fn_id| &analysis.fns[&fn_id])
}

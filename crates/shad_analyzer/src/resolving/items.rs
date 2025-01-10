use crate::registration::constants::{Constant, ConstantId, ConstantValue};
use crate::resolving::types::fn_args;
use crate::{
    errors, registration, Analysis, Buffer, BufferId, FnId, Function, StructField, TypeId,
};
use shad_error::SemanticError;
use shad_parser::{AstFnCall, AstIdent};
use std::iter;
use std::ops::Deref;

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
            analysis.types.get(&registration::types::id(
                analysis,
                &name.label,
                module.map(Deref::deref),
            ))
        })
        .find(|type_| {
            type_.ast.as_ref().map_or(true, |ast| ast.is_pub)
                || type_.module.as_deref() == Some(module)
        })
        .map(|type_| type_.id.clone())
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

pub(crate) fn fn_<'a>(analysis: &'a Analysis, call: &AstFnCall) -> Option<&'a Function> {
    let module = &call.name.span.module.name;
    let arg_types = fn_args(analysis, call)?;
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

pub(crate) fn const_fn<'a>(
    analysis: &'a Analysis,
    call: &AstFnCall,
    args: &[ConstantValue],
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
                param_types: args.iter().map(|arg| Some(arg.type_id())).collect(),
                param_count: args.len(),
                is_generic: false,
            };
            analysis.fns.get(&id)
        })
        .find(|fn_| fn_.ast.is_const && (fn_.ast.is_pub || &fn_.id.module == module))
}

pub(crate) fn field<'a>(
    analysis: &'a Analysis,
    type_id: &TypeId,
    field: &AstIdent,
) -> Option<&'a StructField> {
    let type_module = &analysis.types[type_id].module;
    let field_usage_module = &field.span.module.name;
    analysis.types[type_id]
        .fields
        .iter()
        .filter(|type_field| {
            type_field.is_pub || type_module.as_deref() == Some(field_usage_module)
        })
        .find(|type_field| type_field.name.label == field.label)
}

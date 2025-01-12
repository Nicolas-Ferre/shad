use crate::registration::constants::{Constant, ConstantId, ConstantValue};
use crate::resolving::types::fn_args;
use crate::{
    errors, resolving, Analysis, Buffer, BufferId, FnId, Function, GenericValue, StructField,
    TypeId,
};
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

pub(crate) fn fn_<'a>(analysis: &'a Analysis, call: &AstFnCall, raw: bool) -> Option<&'a Function> {
    let arg_types = fn_args(analysis, call)?;
    let generic_args = resolving::expressions::fn_call_generic_values(analysis, call)?;
    fn_with_genericity(analysis, call, raw, &arg_types, &generic_args, false)
        .or_else(|| fn_with_genericity(analysis, call, raw, &arg_types, &generic_args, true))
}

fn fn_with_genericity<'a>(
    analysis: &'a Analysis,
    call: &AstFnCall,
    raw: bool,
    arg_types: &[TypeId],
    generic_args: &[GenericValue],
    is_generic: bool,
) -> Option<&'a Function> {
    let module = &call.name.span.module.name;
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .filter_map(|module| {
            let id = if is_generic {
                FnId {
                    module: module.clone(),
                    name: call.name.label.clone(),
                    param_types: vec![],
                    generic_values: vec![],
                    param_count: call.args.len(),
                    is_generic: true,
                }
            } else {
                FnId {
                    module: module.clone(),
                    name: call.name.label.clone(),
                    param_types: arg_types.iter().map(|type_| Some(type_.clone())).collect(),
                    generic_values: generic_args.to_vec(),
                    param_count: arg_types.len(),
                    is_generic: false,
                }
            };
            let fns = if raw {
                &analysis.raw_fns
            } else {
                &analysis.fns
            };
            fns.get(&id)
        })
        .find(|fn_| {
            (fn_.ast.is_pub || &fn_.id.module == module)
                && have_same_param_types(analysis, call, fn_)
        })
}

fn have_same_param_types(analysis: &Analysis, call: &AstFnCall, fn_: &Function) -> bool {
    if let (Some(arg_type_ids), Some(param_type_ids)) = (
        fn_args(analysis, call),
        fn_.params
            .iter()
            .map(|param| param.type_id.clone())
            .collect::<Option<Vec<_>>>(),
    ) {
        arg_type_ids == param_type_ids
    } else {
        unreachable!("internal error: invalid function param/argument type");
    }
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
                generic_values: vec![],
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
    let module = &field.span.module.name;
    analysis.types[type_id]
        .fields
        .iter()
        .filter(|type_field| type_field.is_pub || type_id.module.as_deref() == Some(module))
        .find(|type_field| type_field.name.label == field.label)
}

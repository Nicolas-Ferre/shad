use crate::registration::generics::{ConstantGenericParam, GenericParam};
use crate::{errors, Analysis};
use fxhash::FxHashMap;
use shad_error::SemanticError;

const SUPPORTED_CONST_TYPES: &[&str] = &["u32", "i32", "f32", "bool"];

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for type_ in analysis.types.values() {
        check_item_params(&mut errors, &type_.generics);
    }
    for fn_ in analysis.raw_fns.values() {
        check_item_params(&mut errors, &fn_.generics);
    }
    analysis.errors.extend(errors);
}

fn check_item_params(errors: &mut Vec<SemanticError>, generics: &[GenericParam]) {
    for param in generics {
        if let GenericParam::Constant(ConstantGenericParam {
            type_,
            type_id: Some(type_id),
            ..
        }) = param
        {
            if type_id.module.is_some() || !SUPPORTED_CONST_TYPES.contains(&type_id.name.as_str()) {
                let error = errors::constants::unsupported_type(type_);
                errors.push(error);
            }
        }
    }
    let mut name_params = FxHashMap::default();
    for param in generics {
        if let Some(duplicated_param) = name_params.insert(&param.name().label, param) {
            let error = errors::generics::duplicated_param(duplicated_param, param);
            errors.push(error);
        }
    }
}

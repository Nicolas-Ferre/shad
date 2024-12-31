use crate::{errors, Analysis, ConstantGenericParam, GenericParam};

const SUPPORTED_CONST_TYPES: &[&str] = &["u32", "i32", "f32", "bool"];

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for type_ in analysis.types.values() {
        for param in &type_.generics {
            if let GenericParam::Constant(ConstantGenericParam {
                type_name,
                type_id: Some(type_id),
                ..
            }) = param
            {
                if type_id.module.is_some()
                    || !SUPPORTED_CONST_TYPES.contains(&type_id.name.as_str())
                {
                    let error = errors::constants::unsupported_type(type_name);
                    errors.push(error);
                }
            }
        }
    }
    analysis.errors.extend(errors);
}

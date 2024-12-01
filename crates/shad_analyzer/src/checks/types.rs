use crate::{errors, resolver, Analysis};
use shad_parser::{AstStructField, AstStructItem};

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for (struct_, field) in struct_fields(analysis) {
        if let Ok(field_type) = resolver::type_(analysis, &field.type_) {
            if field_type.module.is_some() {
                errors.push(errors::types::invalid_field_type(struct_, &field.type_));
            }
        }
    }
    analysis.errors.extend(errors);
}

fn struct_fields(analysis: &Analysis) -> impl Iterator<Item = (&AstStructItem, &AstStructField)> {
    analysis
        .types
        .values()
        .flat_map(|type_| &type_.ast)
        .flat_map(|struct_| struct_.fields.iter().map(move |field| (struct_, field)))
}

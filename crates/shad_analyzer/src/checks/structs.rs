use crate::registration::types;
use crate::{errors, Analysis};
use shad_parser::{AstStructField, AstStructItem};

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for (struct_, field) in struct_fields(analysis) {
        let module = &struct_.name.span.module.name;
        if let Some(field_type) = types::find(analysis, module, &field.type_) {
            if field_type.module.is_some() {
                errors.push(errors::types::invalid_field_type(struct_, &field.type_));
            }
        } else {
            errors.push(errors::types::not_found(&field.type_));
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

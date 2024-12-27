use crate::registration::types::WGSL_ARRAY_TYPE;
use crate::{errors, Analysis};

pub(crate) fn check(analysis: &mut Analysis) {
    for type_ in analysis.types.values() {
        if let Some(ast) = &type_.ast {
            if let (Some(gpu), None) = (&ast.gpu_qualifier, &ast.layout) {
                let gpu_name = gpu.name.as_ref().map(|name| name.root.label.as_str());
                if gpu_name != Some(WGSL_ARRAY_TYPE) {
                    let error = errors::types::missing_layout(ast, gpu);
                    analysis.errors.push(error);
                }
            }
            if ast.gpu_qualifier.is_none() && ast.fields.is_empty() {
                let error = errors::types::no_field(ast);
                analysis.errors.push(error);
            }
        }
    }
}

use crate::{errors, Analysis};

pub(crate) fn check(analysis: &mut Analysis) {
    for type_ in analysis.types.values() {
        if let Some(ast) = &type_.ast {
            if let (Some(gpu), None) = (&ast.gpu_qualifier, &ast.layout) {
                let error = errors::types::missing_layout(ast, gpu);
                analysis.errors.push(error);
            }
        }
    }
}

use crate::{resolving, Analysis};
use shad_parser::{AstGpuGenericParam, AstGpuName};

pub(crate) fn check(analysis: &mut Analysis) {
    for type_ in analysis.types.clone().values() {
        if let Some(name) = &type_
            .ast
            .as_ref()
            .and_then(|ast| ast.gpu_qualifier.as_ref())
            .and_then(|gpu| gpu.name.as_ref())
        {
            check_name(analysis, name);
        }
    }
    for fn_ in analysis.fns.clone().into_values() {
        let name = fn_
            .ast
            .gpu_qualifier
            .as_ref()
            .and_then(|gpu| gpu.name.as_ref());
        if let Some(name) = name {
            check_name(analysis, name);
        }
    }
}

fn check_name(analysis: &mut Analysis, name: &AstGpuName) {
    for param in &name.generics {
        if let AstGpuGenericParam::Ident(param) = param {
            resolving::items::type_id_or_add_error(analysis, param);
        }
    }
}

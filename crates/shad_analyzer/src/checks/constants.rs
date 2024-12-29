use crate::{errors, resolver, Analysis};
use shad_parser::AstExprRoot;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for constant in analysis.constants.values() {
        if let AstExprRoot::FnCall(call) = &constant.ast.value.root {
            if resolver::fn_(analysis, &call.name).map_or(false, |fn_| !fn_.ast.is_const) {
                let error = errors::constants::non_const_fn_call(call);
                errors.push(error);
            }
        }
    }
    analysis.errors.extend(errors);
}

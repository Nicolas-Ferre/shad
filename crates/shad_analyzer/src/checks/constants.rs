use crate::{errors, Analysis};
use shad_parser::AstExprRoot;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for constant in analysis.constants.values() {
        if matches!(constant.ast.value.root, AstExprRoot::FnCall(_)) {
            let error = errors::constants::invalid_expr(&constant.ast.value);
            errors.push(error);
        }
    }
    analysis.errors.extend(errors);
}

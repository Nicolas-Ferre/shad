use crate::resolving::items::Item;
use crate::{errors, resolving, Analysis};
use shad_parser::AstExprRoot;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for constant in analysis.constants.values() {
        if let AstExprRoot::FnCall(call) = &constant.ast.value.root {
            if let Some(Item::Fn(fn_)) = resolving::items::item(analysis, &call.name) {
                if !fn_.ast.is_const {
                    let error = errors::constants::non_const_fn_call(call);
                    errors.push(error);
                }
            }
        }
    }
    analysis.errors.extend(errors);
}

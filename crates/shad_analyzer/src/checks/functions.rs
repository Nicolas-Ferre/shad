use crate::{errors, Analysis};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{
    AstFnItem, ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN,
    NE_FN, NOT_FN, OR_FN, SUB_FN,
};

const UNARY_FN_PARAM_COUNT: usize = 1;
const BINARY_FN_PARAM_COUNT: usize = 2;
const SPECIAL_UNARY_FNS: [&str; 2] = [NEG_FN, NOT_FN];
const SPECIAL_BINARY_FNS: [&str; 13] = [
    ADD_FN, SUB_FN, MUL_FN, DIV_FN, MOD_FN, EQ_FN, NE_FN, GT_FN, LT_FN, GE_FN, LE_FN, AND_FN, OR_FN,
];

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for fn_ in analysis.fns.values() {
        check_fn(&fn_.ast, &mut errors);
    }
    analysis.errors.extend(errors);
}

fn check_fn(fn_: &AstFnItem, errors: &mut Vec<SemanticError>) {
    errors.extend(check_param_count(
        fn_,
        &SPECIAL_UNARY_FNS,
        UNARY_FN_PARAM_COUNT,
    ));
    errors.extend(check_param_count(
        fn_,
        &SPECIAL_BINARY_FNS,
        BINARY_FN_PARAM_COUNT,
    ));
    errors.extend(check_duplicated_params(fn_));
}

fn check_param_count(
    fn_: &AstFnItem,
    matching_names: &[&str],
    expected_count: usize,
) -> Option<SemanticError> {
    if matching_names.contains(&fn_.name.label.as_str()) && fn_.params.len() != expected_count {
        Some(errors::functions::invalid_param_count(fn_, expected_count))
    } else {
        None
    }
}

fn check_duplicated_params(fn_: &AstFnItem) -> Vec<SemanticError> {
    let mut names = FxHashMap::default();
    fn_.params
        .iter()
        .filter_map(|param| {
            names
                .insert(&param.name.label, &param.name)
                .map(|existing_param| (&param.name, existing_param))
        })
        .map(|(param1, param2)| errors::functions::duplicated_param(param1, param2))
        .collect()
}

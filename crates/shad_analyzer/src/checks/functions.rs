use crate::{errors, Analysis, ConstFn, ConstFnId, Function, TypeId};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{
    ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN,
    NOT_FN, OR_FN, SUB_FN,
};

const UNARY_FN_PARAM_COUNT: usize = 1;
const BINARY_FN_PARAM_COUNT: usize = 2;
const SPECIAL_UNARY_FNS: [&str; 2] = [NEG_FN, NOT_FN];
const SPECIAL_BINARY_FNS: [&str; 13] = [
    ADD_FN, SUB_FN, MUL_FN, DIV_FN, MOD_FN, EQ_FN, NE_FN, GT_FN, LT_FN, GE_FN, LE_FN, AND_FN, OR_FN,
];

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    for fn_ in analysis.raw_fns.values() {
        check_fn(analysis, fn_, &mut errors);
    }
    analysis.errors.extend(errors);
}

fn check_fn(analysis: &Analysis, fn_: &Function, errors: &mut Vec<SemanticError>) {
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
    errors.extend(check_gpu_const_fn(analysis, fn_));
}

fn check_param_count(
    fn_: &Function,
    matching_names: &[&str],
    expected_count: usize,
) -> Option<SemanticError> {
    if matching_names.contains(&fn_.ast.name.label.as_str()) && fn_.params.len() != expected_count {
        Some(errors::functions::invalid_param_count(
            &fn_.ast,
            expected_count,
        ))
    } else {
        None
    }
}

fn check_duplicated_params(fn_: &Function) -> Vec<SemanticError> {
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

fn check_gpu_const_fn(analysis: &Analysis, fn_: &Function) -> Option<SemanticError> {
    if fn_.ast.is_const && fn_.ast.gpu_qualifier.is_some() {
        if let Some((const_fn_id, &const_fn)) = fn_.const_fn_id().and_then(|id| {
            analysis
                .const_functions
                .get(&id)
                .map(|const_fn| (id, const_fn))
        }) {
            if let (Some(return_type), Some(actual_return_type_id)) =
                (&fn_.ast.return_type, &fn_.return_type_id)
            {
                let expected_return_type_id =
                    expected_const_fn_return_type_id(const_fn_id, const_fn);
                if &expected_return_type_id == actual_return_type_id {
                    None
                } else {
                    Some(errors::functions::invalid_const_fn_return_type(
                        fn_,
                        return_type,
                        &expected_return_type_id,
                        actual_return_type_id,
                    ))
                }
            } else {
                None
            }
        } else {
            Some(errors::functions::not_found_const_fn(fn_))
        }
    } else {
        None
    }
}

fn expected_const_fn_return_type_id(const_fn_id: ConstFnId, const_fn: ConstFn) -> TypeId {
    let test_params: Vec<_> = const_fn_id
        .param_types
        .iter()
        .map(|param_type| param_type.zero_value())
        .collect();
    const_fn(&test_params).type_id()
}

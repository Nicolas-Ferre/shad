use crate::atoms;
use itertools::Itertools;
use shad_analyzer::{Analysis, FnParam, Function, TypeId};
use shad_parser::{
    AstExpr, AstFnCall, ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN,
    NEG_FN, NE_FN, NOT_FN, OR_FN, SUB_FN,
};

pub(crate) fn to_wgsl(analysis: &Analysis, call: &AstFnCall) -> String {
    let fn_ = analysis.fn_(call).expect("internal error: missing fn");
    cast_fn_call(
        fn_,
        if let Some(operator) = unary_operator(analysis, fn_) {
            format!(
                "({}{})",
                operator,
                cast_fn_arg(analysis, fn_, &fn_.params[0], &call.args[0].value)
            )
        } else if let Some(operator) = binary_operator(analysis, fn_) {
            format!(
                "({} {} {})",
                cast_fn_arg(analysis, fn_, &fn_.params[0], &call.args[0].value),
                operator,
                cast_fn_arg(analysis, fn_, &fn_.params[1], &call.args[1].value)
            )
        } else {
            format!(
                "{}({})",
                atoms::to_fn_ident_wgsl(analysis, fn_),
                call.args
                    .iter()
                    .zip(&fn_.params)
                    .map(|(arg, param)| cast_fn_arg(analysis, fn_, param, &arg.value))
                    .join(", ")
            )
        },
    )
}

fn cast_fn_call(fn_: &Function, call: String) -> String {
    if fn_.return_type.id == Some(TypeId::from_builtin("bool")) {
        format!("u32({call})")
    } else {
        call
    }
}

fn cast_fn_arg(analysis: &Analysis, fn_: &Function, param: &FnParam, arg: &AstExpr) -> String {
    let expr = atoms::to_expr_wgsl(analysis, arg);
    let type_ = &analysis.types[param
        .type_
        .id
        .as_ref()
        .expect("internal error: invalid param type")];
    if fn_.ast.gpu_qualifier.is_some()
        && fn_.source_type.is_none()
        && type_.id == TypeId::from_builtin("bool")
    {
        format!("bool({expr})")
    } else {
        expr
    }
}

fn unary_operator(analysis: &Analysis, fn_: &Function) -> Option<&'static str> {
    if fn_.ast.gpu_qualifier.is_some() {
        match atoms::to_fn_ident_wgsl(analysis, fn_) {
            n if n == NEG_FN => Some("-"),
            n if n == NOT_FN => Some("!"),
            _ => None,
        }
    } else {
        None
    }
}

fn binary_operator(analysis: &Analysis, fn_: &Function) -> Option<&'static str> {
    if fn_.ast.gpu_qualifier.is_some() {
        match atoms::to_fn_ident_wgsl(analysis, fn_) {
            n if n == ADD_FN => Some("+"),
            n if n == SUB_FN => Some("-"),
            n if n == MUL_FN => Some("*"),
            n if n == DIV_FN => Some("/"),
            n if n == MOD_FN => Some("%"),
            n if n == EQ_FN => Some("=="),
            n if n == NE_FN => Some("!="),
            n if n == GT_FN => Some(">"),
            n if n == LT_FN => Some("<"),
            n if n == GE_FN => Some(">="),
            n if n == LE_FN => Some("<="),
            n if n == AND_FN => Some("&&"),
            n if n == OR_FN => Some("||"),
            _ => None,
        }
    } else {
        None
    }
}

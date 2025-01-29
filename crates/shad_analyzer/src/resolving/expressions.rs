use crate::{registration, resolving, Analysis, GenericValue, Item, TypeId};
use shad_parser::{AstExpr, AstExprRoot, AstFnCall, AstGenericArg, AstGenerics};

pub(crate) fn semantic(analysis: &Analysis, expr: &AstExpr) -> ExprSemantic {
    match &expr.root {
        AstExprRoot::Ident(ident) => match analysis.item(ident) {
            Some(Item::Constant(_)) => ExprSemantic::Value,
            Some(Item::Var(var)) => {
                if var.is_const {
                    ExprSemantic::Value
                } else {
                    ExprSemantic::Ref
                }
            }
            Some(Item::Buffer(_)) => ExprSemantic::Ref,
            None => ExprSemantic::None,
        },
        AstExprRoot::FnCall(call) => {
            if expr.fields.is_empty() {
                if let Some(fn_) = resolving::items::fn_(analysis, call) {
                    fn_.ast
                        .return_type
                        .as_ref()
                        .map_or(ExprSemantic::None, |type_| {
                            if type_.is_ref {
                                ExprSemantic::Ref
                            } else {
                                ExprSemantic::Value
                            }
                        })
                } else {
                    ExprSemantic::None
                }
            } else {
                ExprSemantic::Ref
            }
        }
        AstExprRoot::Literal(_) => ExprSemantic::Value,
    }
}

pub(crate) fn fn_call_generic_values(
    analysis: &Analysis,
    call: &AstFnCall,
    _generic_values: &[(String, GenericValue)],
) -> Vec<GenericValue> {
    call.generics
        .args
        .iter()
        .filter_map(|arg| generic_value(analysis, arg))
        .collect()
}

pub(crate) fn generic_value_types(
    analysis: &Analysis,
    generics: &AstGenerics,
) -> Vec<GenericValueType> {
    generics
        .args
        .iter()
        .map(|arg| generic_value_type(analysis, arg))
        .collect()
}

fn generic_value_type(analysis: &Analysis, arg: &AstGenericArg) -> GenericValueType {
    match arg {
        AstGenericArg::Expr(expr) => generic_type_id(analysis, expr)
            .map(|_| GenericValueType::Type)
            .or_else(|| {
                registration::constants::calculate_const_expr_type(analysis, expr)
                    .map(GenericValueType::Constant)
            })
            .unwrap_or(GenericValueType::None),
        AstGenericArg::Type(type_) => resolving::items::type_id(analysis, type_)
            .ok()
            .map_or(GenericValueType::None, |_| GenericValueType::Type),
    }
}

pub(crate) fn generic_values(
    analysis: &Analysis,
    generics: &AstGenerics,
) -> Vec<Option<GenericValue>> {
    generics
        .args
        .iter()
        .map(|arg| generic_value(analysis, arg))
        .collect()
}

fn generic_value(analysis: &Analysis, arg: &AstGenericArg) -> Option<GenericValue> {
    match arg {
        AstGenericArg::Expr(expr) => generic_type_id(analysis, expr)
            .map(GenericValue::Type)
            .or_else(|| {
                registration::constants::calculate_const_expr_value(analysis, expr)
                    .map(GenericValue::Constant)
            }),
        AstGenericArg::Type(type_) => resolving::items::type_id(analysis, type_)
            .ok()
            .map(GenericValue::Type),
    }
}

fn generic_type_id(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    if let (AstExprRoot::Ident(ident), true) = (&expr.root, expr.fields.is_empty()) {
        resolving::items::type_id(analysis, &ident.clone().into()).ok()
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprSemantic {
    None,
    Ref,
    Value,
}

pub(crate) enum GenericValueType {
    Type,
    Constant(TypeId),
    None,
}

use crate::registration::constants::ConstantValue;
use crate::{
    resolving, Analysis, BufferId, Item, Type, TypeId, BOOL_TYPE, F32_TYPE, I32_TYPE, U32_TYPE,
};
use shad_parser::{AstExpr, AstExprRoot, AstFnCall, AstIdent, AstLiteral, AstLiteralType};

pub(crate) fn fn_args(analysis: &Analysis, call: &AstFnCall) -> Option<Vec<TypeId>> {
    call.args
        .iter()
        .map(|arg| expr(analysis, &arg.value))
        .collect::<Option<Vec<_>>>()
}

pub(crate) fn expr(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    let mut last_type_id = expr_root(analysis, expr);
    for field in &expr.fields {
        if let Some(type_id) = &last_type_id {
            last_type_id = resolving::items::field(analysis, type_id, field)
                .and_then(|field| field.type_id.clone());
        } else {
            return None;
        }
    }
    last_type_id
}

pub(crate) fn expr_root(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    match &expr.root {
        AstExprRoot::Ident(value) => ident(analysis, value),
        AstExprRoot::FnCall(value) => fn_call(analysis, value),
        AstExprRoot::Literal(value) => Some(literal(value)),
    }
}

pub(crate) fn literal(literal: &AstLiteral) -> TypeId {
    match literal.type_ {
        AstLiteralType::F32 => F32_TYPE.into(),
        AstLiteralType::U32 => U32_TYPE.into(),
        AstLiteralType::I32 => I32_TYPE.into(),
        AstLiteralType::Bool => BOOL_TYPE.into(),
    }
}

pub(crate) fn buffer<'a>(analysis: &'a Analysis, buffer_id: &BufferId) -> Option<&'a Type> {
    analysis
        .buffers
        .get(buffer_id)
        .and_then(|buffer| buffer.type_id.as_ref())
        .and_then(|type_| analysis.types.get(type_))
}

fn fn_call(analysis: &Analysis, call: &AstFnCall) -> Option<TypeId> {
    let fn_ = resolving::items::fn_(analysis, call)?;
    fn_.return_type_id.clone()
}

fn ident(analysis: &Analysis, ident: &AstIdent) -> Option<TypeId> {
    match analysis.item(ident) {
        Some(Item::Constant(constant)) => constant.value.as_ref().map(ConstantValue::type_id),
        Some(Item::Buffer(buffer)) => buffer.type_id.clone(),
        Some(Item::Var(_, type_id)) => type_id.clone(),
        None => None,
    }
}

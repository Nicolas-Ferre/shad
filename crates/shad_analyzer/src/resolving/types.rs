use crate::registration::constants::ConstantValue;
use crate::{resolving, Analysis, BufferId, Type, TypeId, BOOL_TYPE, F32_TYPE, I32_TYPE, U32_TYPE};
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
        AstExprRoot::FnCall(value) => ident(analysis, &value.name),
        AstExprRoot::Literal(value) => Some(literal(value)),
    }
}

pub(crate) fn literal(literal: &AstLiteral) -> TypeId {
    match literal.type_ {
        AstLiteralType::F32 => TypeId::from_builtin(F32_TYPE),
        AstLiteralType::U32 => TypeId::from_builtin(U32_TYPE),
        AstLiteralType::I32 => TypeId::from_builtin(I32_TYPE),
        AstLiteralType::Bool => TypeId::from_builtin(BOOL_TYPE),
    }
}

pub(crate) fn buffer<'a>(analysis: &'a Analysis, buffer_id: &BufferId) -> Option<&'a Type> {
    analysis
        .buffers
        .get(buffer_id)
        .map(|buffer| buffer.ast.name.id)
        .and_then(|id| analysis.idents.get(&id))
        .and_then(|ident| ident.type_id.as_ref())
        .and_then(|type_| analysis.types.get(type_))
}

fn ident(analysis: &Analysis, ident: &AstIdent) -> Option<TypeId> {
    if let Some(ident) = analysis.idents.get(&ident.id) {
        ident.type_id.clone()
    } else {
        resolving::items::constant(analysis, ident)
            .and_then(|constant| constant.value.as_ref())
            .map(ConstantValue::type_id)
    }
}

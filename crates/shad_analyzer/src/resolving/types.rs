use crate::registration::constants::ConstantId;
use crate::{Analysis, BufferId, Type, TypeId, BOOL_TYPE, F32_TYPE, I32_TYPE, U32_TYPE};
use shad_parser::{AstExpr, AstExprRoot, AstLiteral, AstLiteralType};

pub(crate) fn expr(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    if expr.fields.is_empty() {
        match &expr.root {
            AstExprRoot::Ident(value) => ident(analysis, value.id),
            AstExprRoot::FnCall(value) => ident(analysis, value.name.id),
            AstExprRoot::Literal(value) => Some(literal(value)),
        }
    } else {
        ident(analysis, expr.fields[expr.fields.len() - 1].id)
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

pub(crate) fn constant<'a>(analysis: &'a Analysis, constant_id: &ConstantId) -> Option<&'a Type> {
    analysis
        .constants
        .get(constant_id)
        .map(|buffer| buffer.ast.name.id)
        .and_then(|id| analysis.idents.get(&id))
        .and_then(|ident| ident.type_id.as_ref())
        .and_then(|type_| analysis.types.get(type_))
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

fn ident(analysis: &Analysis, id: u64) -> Option<TypeId> {
    analysis
        .idents
        .get(&id)
        .and_then(|ident| ident.type_id.clone())
}

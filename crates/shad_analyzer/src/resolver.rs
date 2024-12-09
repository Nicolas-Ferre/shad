use crate::{
    errors, Analysis, BufferId, Function, IdentSource, Type, TypeId, BOOL_TYPE, F32_TYPE, I32_TYPE,
    U32_TYPE,
};
use shad_error::SemanticError;
use shad_parser::{AstExpr, AstIdent, AstLiteralType, AstValue, AstValueRoot};

pub(crate) fn type_or_add_error(analysis: &mut Analysis, type_: &AstIdent) -> Option<TypeId> {
    match self::type_(analysis, type_) {
        Ok(type_id) => Some(type_id),
        Err(error) => {
            analysis.errors.push(error);
            None
        }
    }
}

pub(crate) fn type_(analysis: &Analysis, type_: &AstIdent) -> Result<TypeId, SemanticError> {
    let type_id = TypeId {
        module: Some(type_.span.module.name.clone()),
        name: type_.label.clone(),
    };
    if analysis.types.contains_key(&type_id) {
        return Ok(type_id);
    }
    let builtin_type_id = TypeId::from_builtin(&type_.label);
    if analysis.types.contains_key(&builtin_type_id) {
        Ok(builtin_type_id)
    } else {
        Err(errors::types::not_found(type_))
    }
}

pub(crate) fn expr_type(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    match expr {
        AstExpr::Literal(literal) => Some(match literal.type_ {
            AstLiteralType::F32 => TypeId::from_builtin(F32_TYPE),
            AstLiteralType::U32 => TypeId::from_builtin(U32_TYPE),
            AstLiteralType::I32 => TypeId::from_builtin(I32_TYPE),
            AstLiteralType::Bool => TypeId::from_builtin(BOOL_TYPE),
        }),
        AstExpr::Value(value) => value_type(analysis, value).cloned(),
    }
}

pub(crate) fn value_type<'a>(analysis: &'a Analysis, value: &AstValue) -> Option<&'a TypeId> {
    analysis
        .idents
        .get(&value_id(value))
        .and_then(|ident| ident.type_.as_ref())
}

pub(crate) fn value_root_id(value: &AstValue) -> u64 {
    match &value.root {
        AstValueRoot::Ident(ident) => ident.id,
        AstValueRoot::FnCall(call) => call.name.id,
    }
}

pub(crate) fn buffer_type<'a>(analysis: &'a Analysis, buffer_id: &BufferId) -> Option<&'a Type> {
    analysis
        .buffers
        .get(buffer_id)
        .map(|buffer| buffer.ast.name.id)
        .and_then(|id| analysis.idents.get(&id))
        .and_then(|ident| ident.type_.as_ref())
        .and_then(|type_| analysis.types.get(type_))
}

pub(crate) fn fn_<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Function> {
    analysis
        .idents
        .get(&name.id)
        .map(|ident| match &ident.source {
            IdentSource::Fn(id) => id.clone(),
            IdentSource::Buffer(_) | IdentSource::Var(_) | IdentSource::Field => {
                unreachable!("internal error: retrieve non-function ID")
            }
        })
        .map(|fn_id| &analysis.fns[&fn_id])
}

fn value_id(value: &AstValue) -> u64 {
    if value.fields.is_empty() {
        value_root_id(value)
    } else {
        value.fields[value.fields.len() - 1].id
    }
}

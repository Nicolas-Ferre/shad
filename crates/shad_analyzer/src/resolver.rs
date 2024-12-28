use crate::registration::constants::ConstantId;
use crate::{
    errors, Analysis, BufferId, Function, IdentSource, Type, TypeId, BOOL_TYPE, F32_TYPE, I32_TYPE,
    U32_TYPE,
};
use shad_error::SemanticError;
use shad_parser::{AstExpr, AstExprRoot, AstIdent, AstLiteral, AstLiteralType};
use std::iter;

pub(crate) fn type_or_add_error(
    analysis: &mut Analysis,
    module: &str,
    type_: &AstIdent,
) -> Option<TypeId> {
    match self::type_(analysis, module, type_) {
        Ok(type_id) => Some(type_id),
        Err(error) => {
            analysis.errors.push(error);
            None
        }
    }
}

pub(crate) fn type_(
    analysis: &Analysis,
    module: &str,
    type_: &AstIdent,
) -> Result<TypeId, SemanticError> {
    analysis
        .visible_modules
        .get(module)
        .into_iter()
        .flatten()
        .map(Some)
        .chain(iter::once(None))
        .filter_map(|module| {
            let id = TypeId {
                module: module.cloned(),
                name: type_.label.clone(),
            };
            analysis.types.get(&id).map(|type_| (id, type_))
        })
        .find(|(type_id, type_)| {
            type_.ast.as_ref().map_or(true, |ast| ast.is_pub)
                || type_id.module.as_deref() == Some(module)
        })
        .map(|(type_id, _)| type_id)
        .ok_or_else(|| errors::types::not_found(type_))
}

pub(crate) fn expr_type(analysis: &Analysis, expr: &AstExpr) -> Option<TypeId> {
    if expr.fields.is_empty() {
        match &expr.root {
            AstExprRoot::Ident(ident) => ident_type(analysis, ident.id),
            AstExprRoot::FnCall(call) => ident_type(analysis, call.name.id),
            AstExprRoot::Literal(literal) => Some(literal_type(literal)),
        }
    } else {
        ident_type(analysis, expr.fields[expr.fields.len() - 1].id)
    }
}

pub(crate) fn literal_type(literal: &AstLiteral) -> TypeId {
    match literal.type_ {
        AstLiteralType::F32 => TypeId::from_builtin(F32_TYPE),
        AstLiteralType::U32 => TypeId::from_builtin(U32_TYPE),
        AstLiteralType::I32 => TypeId::from_builtin(I32_TYPE),
        AstLiteralType::Bool => TypeId::from_builtin(BOOL_TYPE),
    }
}

pub(crate) fn expr_root_id(expr: &AstExpr) -> Option<u64> {
    match &expr.root {
        AstExprRoot::Ident(ident) => Some(ident.id),
        AstExprRoot::FnCall(call) => Some(call.name.id),
        AstExprRoot::Literal(_) => None,
    }
}

pub(crate) fn constant_type<'a>(
    analysis: &'a Analysis,
    constant_id: &ConstantId,
) -> Option<&'a Type> {
    analysis
        .constants
        .get(constant_id)
        .map(|buffer| buffer.ast.name.id)
        .and_then(|id| analysis.idents.get(&id))
        .and_then(|ident| ident.type_id.as_ref())
        .and_then(|type_| analysis.types.get(type_))
}

pub(crate) fn buffer_type<'a>(analysis: &'a Analysis, buffer_id: &BufferId) -> Option<&'a Type> {
    analysis
        .buffers
        .get(buffer_id)
        .map(|buffer| buffer.ast.name.id)
        .and_then(|id| analysis.idents.get(&id))
        .and_then(|ident| ident.type_id.as_ref())
        .and_then(|type_| analysis.types.get(type_))
}

pub(crate) fn fn_<'a>(analysis: &'a Analysis, name: &AstIdent) -> Option<&'a Function> {
    analysis
        .idents
        .get(&name.id)
        .map(|ident| match &ident.source {
            IdentSource::Fn(id) => id.clone(),
            IdentSource::Constant(_)
            | IdentSource::Buffer(_)
            | IdentSource::Var(_)
            | IdentSource::Field
            | IdentSource::GenericType => {
                unreachable!("internal error: retrieve non-function ID")
            }
        })
        .map(|fn_id| &analysis.fns[&fn_id])
}

pub(crate) fn expr_semantic(analysis: &Analysis, expr: &AstExpr) -> ExprSemantic {
    match &expr.root {
        AstExprRoot::Ident(ident) => {
            if let Some(ident) = analysis.idents.get(&ident.id) {
                if matches!(ident.source, IdentSource::Constant(_)) {
                    ExprSemantic::Value
                } else {
                    ExprSemantic::Ref
                }
            } else {
                ExprSemantic::None
            }
        }
        AstExprRoot::FnCall(call) => {
            if expr.fields.is_empty() {
                fn_(analysis, &call.name)
                    .and_then(|fn_| fn_.ast.return_type.as_ref())
                    .map_or(ExprSemantic::None, |type_| {
                        if type_.is_ref {
                            ExprSemantic::Ref
                        } else {
                            ExprSemantic::Value
                        }
                    })
            } else {
                ExprSemantic::Ref
            }
        }
        AstExprRoot::Literal(_) => ExprSemantic::Value,
    }
}

fn ident_type(analysis: &Analysis, id: u64) -> Option<TypeId> {
    analysis
        .idents
        .get(&id)
        .and_then(|ident| ident.type_id.clone())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprSemantic {
    None,
    Ref,
    Value,
}

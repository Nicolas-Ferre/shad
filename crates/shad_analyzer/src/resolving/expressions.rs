use crate::{resolving, Analysis, IdentSource};
use shad_parser::{AstExpr, AstExprRoot};

pub(crate) fn root_id(expr: &AstExpr) -> Option<u64> {
    match &expr.root {
        AstExprRoot::Ident(ident) => Some(ident.id),
        AstExprRoot::FnCall(call) => Some(call.name.id),
        AstExprRoot::Literal(_) => None,
    }
}

pub(crate) fn semantic(analysis: &Analysis, expr: &AstExpr) -> ExprSemantic {
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
                resolving::items::registered_fn(analysis, &call.name)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprSemantic {
    None,
    Ref,
    Value,
}

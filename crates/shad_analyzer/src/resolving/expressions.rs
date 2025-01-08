use crate::resolving::items::Item;
use crate::{resolving, Analysis};
use shad_parser::{AstExpr, AstExprRoot};

pub(crate) fn semantic(analysis: &Analysis, expr: &AstExpr) -> ExprSemantic {
    match &expr.root {
        AstExprRoot::Ident(ident) => match resolving::items::item(analysis, ident) {
            Some(Item::Constant(_)) => ExprSemantic::Value,
            Some(_) => ExprSemantic::Ref,
            None => ExprSemantic::None,
        },
        AstExprRoot::FnCall(call) => {
            if expr.fields.is_empty() {
                if let Some(Item::Fn(fn_)) = resolving::items::item(analysis, &call.name) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprSemantic {
    None,
    Ref,
    Value,
}

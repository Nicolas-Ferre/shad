use crate::{errors, Analysis};
use itertools::Itertools;
use shad_parser::{AstFnItem, AstFnQualifier, AstItem};
use std::mem;

/// An analyzed function.
#[derive(Debug, Clone)]
pub struct Function {
    /// The function AST.
    pub ast: AstFnItem,
    /// Whether the function will be inlined.
    pub is_inlined: bool,
}

impl Function {
    /// Whether the function has a `ref` parameter.
    pub fn is_inlined(&self) -> bool {
        is_inlined(&self.ast)
    }
}

/// The unique identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnId {
    /// The module in which the function is defined.
    pub module: String,
    /// The function signature, including the function name and parameter types.
    pub signature: String,
}

impl FnId {
    pub(crate) fn new(fn_: &AstFnItem) -> Self {
        Self {
            module: fn_.name.span.module.name.clone(),
            signature: format!(
                "{}({})",
                fn_.name.label,
                fn_.params.iter().map(|param| &param.type_.label).join(", ")
            ),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for items in &ast.items {
            if let AstItem::Fn(fn_) = items {
                let id = FnId::new(fn_);
                let existing_function = analysis.fns.insert(
                    id.clone(),
                    Function {
                        ast: fn_.clone(),
                        is_inlined: is_inlined(fn_),
                    },
                );
                if let Some(existing_fn) = existing_function {
                    analysis
                        .errors
                        .push(errors::functions::duplicated(&id, fn_, &existing_fn.ast));
                }
            }
        }
    }
    analysis.asts = asts;
}

fn is_inlined(fn_: &AstFnItem) -> bool {
    fn_.qualifier != AstFnQualifier::Gpu && (is_returning_ref(fn_) || has_ref_param(fn_))
}

fn is_returning_ref(fn_: &AstFnItem) -> bool {
    fn_.return_type.as_ref().map_or(false, |type_| type_.is_ref)
}

fn has_ref_param(fn_: &AstFnItem) -> bool {
    fn_.params.iter().any(|param| param.ref_span.is_some())
}

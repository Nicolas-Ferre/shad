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

pub(crate) fn register(analysis: &mut Analysis) {
    let items = mem::take(&mut analysis.ast.items);
    for items in &items {
        if let AstItem::Fn(fn_) = items {
            let signature = signature(fn_);
            let existing_function = analysis.fns.insert(
                signature.clone(),
                Function {
                    ast: fn_.clone(),
                    is_inlined: is_inlined(fn_),
                },
            );
            if let Some(existing_fn) = existing_function {
                analysis.errors.push(errors::functions::duplicated(
                    &signature,
                    fn_,
                    &existing_fn.ast,
                ));
            }
        }
    }
    analysis.ast.items = items;
}

pub(crate) fn signature(fn_: &AstFnItem) -> String {
    format!(
        "{}({})",
        fn_.name.label,
        fn_.params.iter().map(|param| &param.type_.label).join(", ")
    )
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

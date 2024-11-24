use crate::registration::types;
use crate::{errors, Analysis, TypeId};
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
    /// The function name.
    pub name: String,
    /// The function parameter types.
    pub param_types: Vec<Option<TypeId>>,
}

impl FnId {
    pub(crate) fn from_item(analysis: &Analysis, fn_: &AstFnItem) -> Self {
        Self {
            module: fn_.name.span.module.name.clone(),
            name: fn_.name.label.clone(),
            param_types: fn_
                .params
                .iter()
                .map(|param| types::find(analysis, &fn_.name.span.module.name, &param.type_))
                .collect(),
        }
    }

    pub(crate) fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.param_types
                .iter()
                .map(|type_| type_.as_ref().map_or("?", |t| t.name.as_str()))
                .join(", ")
        )
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for items in &ast.items {
            if let AstItem::Fn(fn_) = items {
                let id = FnId::from_item(analysis, fn_);
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

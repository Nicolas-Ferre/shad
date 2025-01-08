use crate::{errors, resolving, Analysis, TypeId};
use shad_parser::{AstConstItem, AstExpr, AstExprRoot, AstItem, AstLiteralType};
use std::mem;
use std::str::FromStr;

/// An analyzed constant.
#[derive(Debug, Clone)]
pub struct Constant {
    /// The constant AST.
    pub ast: AstConstItem,
    /// The unique identifier of the constant.
    pub id: ConstantId,
    /// The value of the constant.
    pub value: Option<ConstantValue>,
}

/// The unique identifier of a constant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstantId {
    /// The module in which the constant is defined.
    pub module: String,
    /// The constant name.
    pub name: String,
}

impl ConstantId {
    pub(crate) fn new(constant: &AstConstItem) -> Self {
        Self {
            module: constant.name.span.module.name.clone(),
            name: constant.name.label.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    U32(u32),
    I32(i32),
    F32(f32),
    Bool(bool),
}

impl ConstantValue {
    // coverage: off (simple logic)
    pub(crate) fn type_id(&self) -> TypeId {
        match self {
            Self::U32(_) => TypeId::from_builtin("u32"),
            Self::I32(_) => TypeId::from_builtin("i32"),
            Self::F32(_) => TypeId::from_builtin("f32"),
            Self::Bool(_) => TypeId::from_builtin("bool"),
        }
    }
    // coverage: on
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_items(analysis);
    register_values(analysis);
}

fn register_items(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for item in &ast.items {
            if let AstItem::Const(constant) = item {
                let id = ConstantId::new(constant);
                let constant_details = Constant {
                    ast: constant.clone(),
                    id: id.clone(),
                    value: None,
                };
                let existing_constant = analysis.constants.insert(id, constant_details);
                if let Some(existing_constant) = existing_constant {
                    analysis
                        .errors
                        .push(errors::constants::duplicated(constant, &existing_constant));
                }
            }
        }
    }
    analysis.asts = asts;
}

fn register_values(analysis: &mut Analysis) {
    let mut last_calculated_constant_count = calculated_constant_count(analysis);
    while last_calculated_constant_count < analysis.constants.len() {
        let constant_ids = analysis.constants.keys().cloned().collect::<Vec<_>>();
        for id in constant_ids {
            let constant = &analysis.constants[&id];
            if constant.value.is_none() {
                if !constant.ast.value.fields.is_empty() {
                    continue;
                }
                analysis
                    .constants
                    .get_mut(&id)
                    .expect("internal error: missing constant")
                    .value = calculate_const_expr(analysis, &constant.ast.value);
            }
        }
        let calculated_constant_value = calculated_constant_count(analysis);
        if calculated_constant_value == last_calculated_constant_count {
            break; // recursive constant init
        }
        last_calculated_constant_count = calculated_constant_value;
    }
}

fn calculate_const_expr(analysis: &Analysis, expr: &AstExpr) -> Option<ConstantValue> {
    match &expr.root {
        AstExprRoot::Literal(literal) => {
            let value = &literal.cleaned_value;
            match literal.type_ {
                AstLiteralType::F32 => f32::from_str(value).ok().map(ConstantValue::F32),
                AstLiteralType::U32 => u32::from_str(&value[..value.len() - 1])
                    .ok()
                    .map(ConstantValue::U32),
                AstLiteralType::I32 => i32::from_str(value).ok().map(ConstantValue::I32),
                AstLiteralType::Bool => Some(ConstantValue::Bool(value == "true")),
            }
        }
        AstExprRoot::Ident(ident) => {
            resolving::items::constant(analysis, ident).and_then(|constant| constant.value.clone())
        }
        AstExprRoot::FnCall(call) => {
            let args: Vec<_> = call
                .args
                .iter()
                .map(|arg| calculate_const_expr(analysis, &arg.value))
                .collect::<Option<_>>()?;
            if let Some(fn_) = resolving::items::const_fn(analysis, call, &args) {
                let const_fn_id = fn_.const_fn_id()?;
                analysis
                    .const_functions
                    .get(&const_fn_id)
                    .map(|const_fn| const_fn(&args))
            } else {
                None
            }
        }
    }
}

fn calculated_constant_count(analysis: &Analysis) -> usize {
    analysis
        .constants
        .values()
        .filter(|constant| constant.value.is_some())
        .count()
}

use crate::{errors, Analysis, IdentSource};
use shad_parser::{AstConstItem, AstExpr, AstExprRoot, AstItem, AstLiteralType};
use std::mem;
use std::str::FromStr;

/// An analyzed constant.
#[derive(Debug, Clone)]
pub struct Constant {
    /// The constant AST.
    pub ast: AstConstItem,
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

pub(crate) fn register(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for item in &ast.items {
            if let AstItem::Const(constant) = item {
                let constant_details = Constant {
                    ast: constant.clone(),
                    value: None,
                };
                let existing_constant = analysis
                    .constants
                    .insert(ConstantId::new(constant), constant_details);
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

pub(crate) fn calculate(analysis: &mut Analysis) {
    let mut last_calculated_constant_count = calculated_constant_count(analysis);
    while last_calculated_constant_count < analysis.constants.len() {
        let constant_ids = analysis.constants.keys().cloned().collect::<Vec<_>>();
        for id in constant_ids {
            let constant = &analysis.constants[&id];
            if constant.value.is_none() {
                if !constant.ast.value.fields.is_empty() {
                    return;
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
        AstExprRoot::Literal(literal) => match literal.type_ {
            AstLiteralType::F32 => f32::from_str(&literal.value).ok().map(ConstantValue::F32),
            AstLiteralType::U32 => u32::from_str(&literal.value[..literal.value.len() - 1])
                .ok()
                .map(ConstantValue::U32),
            AstLiteralType::I32 => i32::from_str(&literal.value).ok().map(ConstantValue::I32),
            AstLiteralType::Bool => Some(ConstantValue::Bool(literal.value == "true")),
        },
        AstExprRoot::Ident(ident) => {
            if let Some(ident) = &analysis.idents.get(&ident.id) {
                if let IdentSource::Constant(constant_id) = &ident.source {
                    analysis
                        .constants
                        .get(constant_id)
                        .as_ref()
                        .and_then(|constant| constant.value.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
        AstExprRoot::FnCall(_) => None,
    }
}

fn calculated_constant_count(analysis: &Analysis) -> usize {
    analysis
        .constants
        .values()
        .filter(|constant| constant.value.is_some())
        .count()
}

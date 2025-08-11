use crate::config::ValidationConfig;
use crate::validation::ValidationContext;
use crate::{AstNode, ValidationError, ValidationErrorLevel};
use std::str::FromStr;

pub(crate) fn check_number_range(
    ctx: &mut ValidationContext<'_>,
    validation: &ValidationConfig,
    node: &AstNode,
) {
    let type_ = &validation.params["type"];
    let slice = node.slice.replace("_", "").replace("u", "");
    let is_invalid_range = match type_.as_str() {
        "i32" => i32::from_str(&slice).is_err(),
        "u32" => u32::from_str(&slice).is_err(),
        "f32" => match f32::from_str(&slice) {
            Ok(value) => value.is_infinite(),
            Err(_) => true,
        },
        _ => unreachable!("undefined `{type_}` number type"),
    };
    if is_invalid_range {
        ctx.errors.push(ValidationError {
            level: ValidationErrorLevel::Error,
            message: format!("out of bound `{type_}` literal"),
            span: node.span(),
            code: ctx.asts[ctx.path].code.clone(),
            path: ctx.path.into(),
            inner: vec![],
        });
    }
}

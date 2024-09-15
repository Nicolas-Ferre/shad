use crate::{type_, Asg, AsgExpr, AsgType};
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstFnParam, AstGpuFnItem, AstIdent};
use std::rc::Rc;

/// An analyzed function signature.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AsgFnSignature {
    /// The function name
    pub name: String,
    /// The function parameter types.
    pub param_types: Vec<String>,
}

impl AsgFnSignature {
    pub(crate) fn new(fn_: &AsgFn) -> Self {
        Self {
            name: fn_.name.label.clone(),
            param_types: fn_
                .params
                .iter()
                .map(|param| param.type_.name().into())
                .collect(),
        }
    }

    pub(crate) fn from_call(asg: &Asg, name: &AstIdent, args: &[AsgExpr]) -> Self {
        Self {
            name: name.label.clone(),
            param_types: args
                .iter()
                .map(|arg| arg.type_(asg).name().into())
                .collect(),
        }
    }
}

/// An analyzed function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFn {
    /// The function name in the initial Shad code.
    pub name: AstIdent,
    /// The function name in the initial Shad code.
    pub params: Vec<AsgFnParam>,
    /// The function returned type.
    pub return_type: Rc<AsgType>,
}

impl AsgFn {
    pub(crate) fn new(asg: &mut Asg, fn_: &AstGpuFnItem) -> Self {
        Self::check_duplicated_params(asg, fn_);
        Self {
            name: fn_.name.clone(),
            params: fn_
                .params
                .iter()
                .map(|param| AsgFnParam::new(asg, param))
                .collect(),
            return_type: type_::find(asg, &fn_.return_type).clone(),
        }
    }

    fn check_duplicated_params(asg: &mut Asg, fn_: &AstGpuFnItem) {
        let mut names = FxHashMap::default();
        for param in &fn_.params {
            let existing = names.insert(&param.name.label, &param.name);
            if let Some(existing) = existing {
                asg.errors
                    .push(Self::duplicated_param_error(asg, &param.name, existing));
            }
        }
    }

    fn duplicated_param_error(
        asg: &Asg,
        duplicated_param_name: &AstIdent,
        existing_param_name: &AstIdent,
    ) -> SemanticError {
        SemanticError::new(
            format!(
                "parameter `{}` is defined multiple times",
                &duplicated_param_name.label,
            ),
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: duplicated_param_name.span,
                    text: "duplicated parameter".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: existing_param_name.span,
                    text: "parameter with same name is defined here".into(),
                },
            ],
            &asg.code,
            &asg.path,
        )
    }
}

/// An analyzed function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnParam {
    /// The parameter name in the initial Shad code.
    pub name: AstIdent,
    /// The parameter type.
    pub type_: Rc<AsgType>,
}

impl AsgFnParam {
    fn new(asg: &mut Asg, param: &AstFnParam) -> Self {
        Self {
            name: param.name.clone(),
            type_: type_::find(asg, &param.type_).clone(),
        }
    }
}

pub(crate) fn find<'a>(
    asg: &'a mut Asg,
    name: &AstIdent,
    signature: &AsgFnSignature,
) -> Option<&'a Rc<AsgFn>> {
    if let Some(function) = asg.functions.get(signature) {
        Some(function)
    } else {
        asg.errors.extend(not_found_error(asg, name, signature));
        None
    }
}

pub(crate) fn duplicated_error(
    asg: &Asg,
    duplicated_fn: &AstGpuFnItem,
    existing_fn: &AsgFn,
) -> Option<SemanticError> {
    let has_invalid_parameter = existing_fn
        .params
        .iter()
        .any(|param| &param.type_ == type_::undefined(asg));
    (!has_invalid_parameter).then(|| {
        SemanticError::new(
            format!(
                "function with signature `{}({})` is defined multiple times",
                &duplicated_fn.name.label,
                duplicated_fn
                    .params
                    .iter()
                    .map(|param| param.type_.label.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: duplicated_fn.name.span,
                    text: "duplicated function".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: existing_fn.name.span,
                    text: "function with same signature is defined here".into(),
                },
            ],
            &asg.code,
            &asg.path,
        )
    })
}

fn not_found_error(
    asg: &Asg,
    name: &AstIdent,
    signature: &AsgFnSignature,
) -> Option<SemanticError> {
    let has_invalid_parameter = signature
        .param_types
        .iter()
        .any(|type_| type_.starts_with('<'));
    (!has_invalid_parameter).then(|| {
        SemanticError::new(
            format!(
                "could not find `{}({})` function",
                signature.name,
                signature.param_types.join(", ")
            ),
            vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: name.span,
                text: "undefined function".into(),
            }],
            &asg.code,
            &asg.path,
        )
    })
}
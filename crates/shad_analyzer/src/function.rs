use crate::statement::{AsgStatement, AsgStatementScopeType, AsgStatements};
use crate::{type_, Asg, AsgExpr, AsgType};
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstFnItem, AstFnParam, AstFnQualifier, AstIdent};
use std::rc::Rc;

/// The function name corresponding to unary `-` operator behavior.
pub const NEG_FN: &str = "__neg__";
/// The function name corresponding to unary `!` operator behavior.
pub const NOT_FN: &str = "__not__";
/// The function name corresponding to binary `+` operator behavior.
pub const ADD_FN: &str = "__add__";
/// The function name corresponding to binary `-` operator behavior.
pub const SUB_FN: &str = "__sub__";
/// The function name corresponding to binary `*` operator behavior.
pub const MUL_FN: &str = "__mul__";
/// The function name corresponding to binary `/` operator behavior.
pub const DIV_FN: &str = "__div__";
/// The function name corresponding to binary `%` operator behavior.
pub const MOD_FN: &str = "__mod__";
/// The function name corresponding to binary `==` operator behavior.
pub const EQ_FN: &str = "__eq__";
/// The function name corresponding to binary `!=` operator behavior.
pub const NE_FN: &str = "__ne__";
/// The function name corresponding to binary `>` operator behavior.
pub const GT_FN: &str = "__gt__";
/// The function name corresponding to binary `<` operator behavior.
pub const LT_FN: &str = "__lt__";
/// The function name corresponding to binary `>=` operator behavior.
pub const GE_FN: &str = "__ge__";
/// The function name corresponding to binary `<=` operator behavior.
pub const LE_FN: &str = "__le__";
/// The function name corresponding to binary `&&` operator behavior.
pub const AND_FN: &str = "__and__";
/// The function name corresponding to binary `||` operator behavior.
pub const OR_FN: &str = "__or__";
const SPECIAL_UNARY_FNS: [&str; 2] = [NEG_FN, NOT_FN];
const SPECIAL_BINARY_FNS: [&str; 13] = [
    ADD_FN, SUB_FN, MUL_FN, DIV_FN, MOD_FN, EQ_FN, NE_FN, GT_FN, LT_FN, GE_FN, LE_FN, AND_FN, OR_FN,
];

/// An analyzed function signature.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AsgFnSignature {
    /// The function name
    pub name: String,
    /// The function parameter types.
    pub param_types: Vec<String>,
}

impl AsgFnSignature {
    pub(crate) fn new(fn_: &AstFnItem) -> Self {
        Self {
            name: fn_.name.label.clone(),
            param_types: fn_
                .params
                .iter()
                .map(|param| param.type_.label.clone())
                .collect(),
        }
    }

    pub(crate) fn from_call(asg: &Asg, name: &str, args: &[AsgExpr]) -> Result<Self, ()> {
        Ok(Self {
            name: name.to_string(),
            param_types: args
                .iter()
                .map(|arg| {
                    let self1 = &arg.type_(asg)?;
                    Ok(self1.name.as_str().into())
                })
                .collect::<Result<_, ()>>()?,
        })
    }
}

/// An analyzed function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFn {
    /// The parsed function.
    pub ast: AstFnItem,
    /// The function signature.
    pub signature: AsgFnSignature,
    /// The unique function index.
    pub index: usize,
    /// The function name in the initial Shad code.
    pub params: Vec<Rc<AsgFnParam>>,
    /// The function returned type.
    pub return_type: Result<Option<Rc<AsgType>>, ()>,
}

impl AsgFn {
    pub(crate) fn new(asg: &mut Asg, fn_: &AstFnItem) -> Self {
        Self::check_duplicated_params(asg, fn_);
        if SPECIAL_UNARY_FNS.contains(&fn_.name.label.as_str()) {
            Self::check_unary_fn(asg, fn_);
        }
        if SPECIAL_BINARY_FNS.contains(&fn_.name.label.as_str()) {
            Self::check_binary_fn(asg, fn_);
        }
        let params: Vec<_> = fn_
            .params
            .iter()
            .map(|param| Rc::new(AsgFnParam::new(asg, param)))
            .collect();
        Self {
            ast: fn_.clone(),
            signature: AsgFnSignature::new(fn_),
            index: asg.functions.len(),
            params,
            return_type: if let Some(type_) = &fn_.return_type {
                type_::find(asg, type_).cloned().map(Some)
            } else {
                Ok(None)
            },
        }
    }

    fn check_duplicated_params(asg: &mut Asg, fn_: &AstFnItem) {
        let mut names = FxHashMap::default();
        for param in &fn_.params {
            let existing = names.insert(&param.name.label, &param.name);
            if let Some(existing) = existing {
                asg.errors
                    .push(Self::duplicated_param_error(asg, &param.name, existing));
            }
        }
    }

    fn check_unary_fn(asg: &mut Asg, fn_: &AstFnItem) {
        const EXPECTED_PARAM_COUNT: usize = 1;
        if fn_.params.len() != EXPECTED_PARAM_COUNT {
            asg.errors.push(Self::invalid_param_count_error(
                asg,
                fn_,
                EXPECTED_PARAM_COUNT,
            ));
        }
    }

    fn check_binary_fn(asg: &mut Asg, fn_: &AstFnItem) {
        const EXPECTED_PARAM_COUNT: usize = 2;
        if fn_.params.len() != EXPECTED_PARAM_COUNT {
            asg.errors.push(Self::invalid_param_count_error(
                asg,
                fn_,
                EXPECTED_PARAM_COUNT,
            ));
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

    fn invalid_param_count_error(
        asg: &Asg,
        fn_: &AstFnItem,
        expected_count: usize,
    ) -> SemanticError {
        SemanticError::new(
            format!(
                "function `{}` has an invalid number of parameters",
                fn_.name.label,
            ),
            vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: fn_.name.span,
                text: format!(
                    "found {} parameters, expected {expected_count}",
                    fn_.params.len()
                ),
            }],
            &asg.code,
            &asg.path,
        )
    }
}

/// An analyzed function body.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnBody {
    /// The statements in the function body.
    pub statements: Vec<AsgStatement>,
}

impl AsgFnBody {
    pub(crate) fn new(asg: &mut Asg, fn_: &AsgFn) -> Self {
        Self {
            statements: AsgStatements::parse(
                asg,
                &fn_.ast.statements,
                match fn_.ast.qualifier {
                    AstFnQualifier::None | AstFnQualifier::Gpu => {
                        AsgStatementScopeType::FnBody(fn_)
                    }
                    AstFnQualifier::Buf => AsgStatementScopeType::BufFnBody(fn_),
                },
            ),
        }
    }
}

/// An analyzed function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnParam {
    /// The parameter name in the initial Shad code.
    pub name: AstIdent,
    /// The parameter type.
    pub type_: Result<Rc<AsgType>, ()>,
}

impl AsgFnParam {
    fn new(asg: &mut Asg, param: &AstFnParam) -> Self {
        Self {
            name: param.name.clone(),
            type_: type_::find(asg, &param.type_).cloned(),
        }
    }
}

pub(crate) fn find<'a>(
    asg: &'a mut Asg,
    span: Span,
    signature: &AsgFnSignature,
) -> Result<&'a Rc<AsgFn>, ()> {
    if let Some(function) = asg.functions.get(signature) {
        Ok(function)
    } else {
        asg.errors.push(not_found_error(asg, span, signature));
        Err(())
    }
}

pub(crate) fn signature_str(fn_: &AstFnItem) -> String {
    format!(
        "{}({})",
        &fn_.name.label,
        fn_.params
            .iter()
            .map(|param| param.type_.label.clone())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(crate) fn duplicated_error(
    asg: &Asg,
    duplicated_fn: &AstFnItem,
    existing_fn: &AsgFn,
) -> SemanticError {
    SemanticError::new(
        format!(
            "function `{}` is defined multiple times",
            signature_str(duplicated_fn)
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_fn.name.span,
                text: "duplicated function".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_fn.ast.name.span,
                text: "function with same signature is defined here".into(),
            },
        ],
        &asg.code,
        &asg.path,
    )
}

fn not_found_error(asg: &Asg, span: Span, signature: &AsgFnSignature) -> SemanticError {
    SemanticError::new(
        format!(
            "could not find `{}({})` function",
            signature.name,
            signature.param_types.join(", ")
        ),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span,
            text: "undefined function".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

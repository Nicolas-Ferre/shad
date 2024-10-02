use crate::statement::{AsgStatement, AsgStatementScopeType, AsgStatements};
use crate::{errors, type_, Asg, AsgExpr, AsgType, Error, Result, TypeResolving};
use fxhash::FxHashMap;
use shad_error::Span;
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

    pub(crate) fn from_call(asg: &Asg, name: &str, args: &[AsgExpr]) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            param_types: args
                .iter()
                .map(|arg| {
                    let self1 = &arg.type_(asg)?;
                    Ok(self1.name.as_str().into())
                })
                .collect::<Result<_>>()?,
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
    pub return_type: Result<Option<Rc<AsgType>>>,
}

impl AsgFn {
    pub(crate) fn new(asg: &mut Asg, fn_: &AstFnItem) -> Self {
        // TODO: move in check phase
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

    // TODO: move in check phase
    fn check_duplicated_params(asg: &mut Asg, fn_: &AstFnItem) {
        let mut names = FxHashMap::default();
        for param in &fn_.params {
            let existing = names.insert(&param.name.label, &param.name);
            if let Some(existing) = existing {
                asg.errors
                    .push(errors::fn_::duplicated_param(asg, &param.name, existing));
            }
        }
    }

    // TODO: move in check phase
    fn check_unary_fn(asg: &mut Asg, fn_: &AstFnItem) {
        const EXPECTED_PARAM_COUNT: usize = 1;
        if fn_.params.len() != EXPECTED_PARAM_COUNT {
            asg.errors.push(errors::fn_::invalid_param_count(
                asg,
                fn_,
                EXPECTED_PARAM_COUNT,
            ));
        }
    }

    // TODO: move in check phase
    fn check_binary_fn(asg: &mut Asg, fn_: &AstFnItem) {
        const EXPECTED_PARAM_COUNT: usize = 2;
        if fn_.params.len() != EXPECTED_PARAM_COUNT {
            asg.errors.push(errors::fn_::invalid_param_count(
                asg,
                fn_,
                EXPECTED_PARAM_COUNT,
            ));
        }
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
            statements: AsgStatements::analyze(
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
    pub type_: Result<Rc<AsgType>>,
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
) -> Result<&'a Rc<AsgFn>> {
    if let Some(function) = asg.functions.get(signature) {
        Ok(function)
    } else {
        asg.errors
            .push(errors::fn_::not_found(asg, span, signature));
        Err(Error)
    }
}

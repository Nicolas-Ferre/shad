use crate::items::statement::StatementContext;
use crate::passes::check::StatementScope;
use crate::{Asg, AsgExpr, AsgStatement, AsgType, Result, TypeResolving};
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
pub(crate) const SPECIAL_UNARY_FNS: [&str; 2] = [NEG_FN, NOT_FN];
pub(crate) const SPECIAL_BINARY_FNS: [&str; 13] = [
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
                asg.find_type(type_).cloned().map(Some)
            } else {
                Ok(None)
            },
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
    pub(crate) fn new(asg: &mut Asg, fn_: &Rc<AsgFn>) -> Self {
        Self {
            statements: StatementContext::analyze(
                asg,
                &fn_.ast.statements,
                match fn_.ast.qualifier {
                    AstFnQualifier::None | AstFnQualifier::Gpu => {
                        StatementScope::FnBody(fn_.clone())
                    }
                    AstFnQualifier::Buf => StatementScope::BufFnBody(fn_.clone()),
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
            type_: asg.find_type(&param.type_).cloned(),
        }
    }
}

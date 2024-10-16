use crate::items::statement::StatementContext;
use crate::passes::check::StatementScope;
use crate::{Asg, AsgExpr, AsgStatement, AsgType, Result, TypeResolving};
use shad_parser::{AstFnItem, AstFnParam, AstFnQualifier};
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
                    let type_ = &arg.type_(asg)?;
                    Ok(type_.name.as_str().into())
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
            .enumerate()
            .map(|(index, param)| Rc::new(AsgFnParam::new(asg, param, index)))
            .collect();
        Self {
            ast: fn_.clone(),
            signature: AsgFnSignature::new(fn_),
            index: asg.functions.len(),
            params,
            return_type: if let Some(type_) = &fn_.return_type {
                asg.find_type(&type_.name).cloned().map(Some)
            } else {
                Ok(None)
            },
        }
    }

    /// Whether the function has a `ref` parameter.
    pub fn is_inlined(&self) -> bool {
        self.ast.qualifier != AstFnQualifier::Gpu
            && (self.is_returning_ref()
                || self.params.iter().any(|param| param.ast.ref_span.is_some()))
    }

    pub(crate) fn is_returning_ref(&self) -> bool {
        self.ast
            .return_type
            .as_ref()
            .map_or(false, |type_| type_.is_ref)
    }

    pub(crate) fn scope(self: &Rc<Self>) -> StatementScope {
        match self.ast.qualifier {
            AstFnQualifier::None | AstFnQualifier::Gpu => StatementScope::FnBody(self.clone()),
            AstFnQualifier::Buf => StatementScope::BufFnBody(self.clone()),
        }
    }
}

/// An analyzed function body.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnBody {
    /// The analyzed function.
    pub fn_: Rc<AsgFn>,
    /// The statements in the function body.
    pub statements: Vec<AsgStatement>,
}

impl AsgFnBody {
    pub(crate) fn new(asg: &mut Asg, fn_: &Rc<AsgFn>) -> Self {
        Self {
            fn_: fn_.clone(),
            statements: StatementContext::analyze(asg, &fn_.ast.statements, fn_.scope()),
        }
    }
}

/// An analyzed function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgFnParam {
    /// The parsed parameter.
    pub ast: AstFnParam,
    /// The parameter type.
    pub type_: Result<Rc<AsgType>>,
    /// The parameter unique index in the function.
    pub index: usize,
}

impl AsgFnParam {
    fn new(asg: &mut Asg, param: &AstFnParam, index: usize) -> Self {
        Self {
            ast: param.clone(),
            type_: asg.find_type(&param.type_).cloned(),
            index,
        }
    }
}

use crate::result::result_ref;
use crate::{Asg, AsgExpr, AsgFnCall, AsgIdent, AsgType};
use crate::{AsgIdentSource, Result};
use std::rc::Rc;

/// A trait implemented to resolve the type of an expression.
pub trait TypeResolving {
    /// Returns the expression type.
    ///
    /// # Errors
    ///
    /// An error is returned if the type cannot be determined due to a previous error.
    fn type_<'a>(&'a self, asg: &'a Asg) -> Result<&'a Rc<AsgType>>;
}

impl TypeResolving for AsgExpr {
    fn type_<'a>(&'a self, asg: &'a Asg) -> Result<&'a Rc<AsgType>> {
        match self {
            Self::Literal(literal) => Ok(&literal.type_),
            Self::Ident(ident) => ident.type_(asg),
            Self::FnCall(call) => call.type_(asg),
        }
    }
}

impl TypeResolving for AsgIdent {
    fn type_<'a>(&'a self, asg: &'a Asg) -> Result<&'a Rc<AsgType>> {
        match &self.source {
            AsgIdentSource::Buffer(buffer) => result_ref(&buffer.expr)?.type_(asg),
            AsgIdentSource::Var(var) => result_ref(&var.expr)?.type_(asg),
            AsgIdentSource::Param(param) => result_ref(&param.type_),
        }
    }
}

impl TypeResolving for AsgFnCall {
    fn type_<'a>(&'a self, _asg: &'a Asg) -> Result<&'a Rc<AsgType>> {
        Ok(result_ref(&self.fn_.return_type)?
            .as_ref()
            .expect("internal error: function call in expression without return type"))
    }
}

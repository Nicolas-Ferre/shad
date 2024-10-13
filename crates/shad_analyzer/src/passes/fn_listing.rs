use crate::result::result_ref;
use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFn, AsgFnCall, AsgLeftValue, AsgStatement,
    AsgVariableDefinition,
};
use fxhash::FxHashMap;
use std::iter;
use std::rc::Rc;

/// A trait implemented to list functions in a code fragment.
pub trait FunctionListing: Sized {
    /// Returns the used functions.
    fn functions<'a>(&'a self, asg: &'a Asg) -> Vec<Rc<AsgFn>>;

    /// Returns the used functions from a slice.
    fn slice_functions(slice: &[Self], asg: &Asg) -> Vec<Rc<AsgFn>> {
        slice
            .iter()
            .flat_map(|statement| statement.functions(asg))
            .map(|fn_| (fn_.index, fn_))
            .collect::<FxHashMap<_, _>>()
            .into_values()
            .collect()
    }
}

impl FunctionListing for AsgStatement {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        match self {
            Self::Assignment(statement) => statement.functions(asg),
            Self::Var(statement) => statement.functions(asg),
            Self::Return(statement) => statement
                .expr
                .as_ref()
                .map(|expr| expr.functions(asg))
                .unwrap_or_default(),
            Self::FnCall(statement) => result_ref(statement)
                .map(|call| call.functions(asg))
                .unwrap_or_default(),
        }
    }
}

impl FunctionListing for AsgAssignment {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        let assigned_fns = result_ref(&self.assigned)
            .map(|expr| expr.functions(asg))
            .unwrap_or_default();
        let expr_fns = result_ref(&self.expr)
            .map(|expr| expr.functions(asg))
            .unwrap_or_default();
        [assigned_fns, expr_fns].concat()
    }
}

impl FunctionListing for AsgLeftValue {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        match self {
            Self::Ident(_) => vec![],
            Self::FnCall(call) => call.functions(asg),
        }
    }
}

impl FunctionListing for AsgVariableDefinition {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        result_ref(&self.expr)
            .map(|expr| expr.functions(asg))
            .unwrap_or_default()
    }
}

impl FunctionListing for AsgExpr {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        match self {
            Self::Literal(_) | Self::Ident(_) => vec![],
            Self::FnCall(expr) => expr.functions(asg),
        }
    }
}

impl FunctionListing for AsgFnCall {
    fn functions(&self, asg: &Asg) -> Vec<Rc<AsgFn>> {
        let arg_fns = self.args.iter().flat_map(|arg| arg.functions(asg));
        let body_fns = asg.function_bodies[self.fn_.index]
            .statements
            .iter()
            .flat_map(|statement| statement.functions(asg));
        let fn_fns = iter::once(self.fn_.clone());
        arg_fns.chain(body_fns).chain(fn_fns).collect()
    }
}

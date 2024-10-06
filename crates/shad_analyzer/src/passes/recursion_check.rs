use crate::errors::fn_;
use crate::result::result_ref;
use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFn, AsgFnCall, AsgReturn, AsgStatement, AsgVariableDefinition,
    Error,
};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use std::rc::Rc;

pub(crate) fn check_recursion(asg: &Asg) -> Vec<SemanticError> {
    let mut checker = FnRecursionChecker::default();
    for fn_ in asg.functions.values() {
        checker.current_fn = Some(fn_.clone());
        checker.calls.clear();
        let _ = fn_.check_recursion(asg, &mut checker);
    }
    checker.errors
}

/// A trait implemented to check function recursion.
trait RecursionCheck {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()>;
}

impl RecursionCheck for AsgFn {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        ctx.check(asg)?;
        for statement in &asg.function_bodies[self.index].statements {
            statement.check_recursion(asg, ctx)?;
        }
        Ok(())
    }
}

impl RecursionCheck for AsgStatement {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        match self {
            Self::Var(statement) => statement.check_recursion(asg, ctx)?,
            Self::Assignment(statement) => statement.check_recursion(asg, ctx)?,
            Self::Return(statement) => statement.check_recursion(asg, ctx)?,
            Self::FnCall(Ok(statement)) => statement.check_recursion(asg, ctx)?,
            Self::FnCall(Err(_)) => (), // no-coverage (failed analysis)
        }
        Ok(())
    }
}

impl RecursionCheck for AsgAssignment {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        result_ref(&self.expr).and_then(|expr| expr.check_recursion(asg, ctx))
    }
}

impl RecursionCheck for AsgVariableDefinition {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        result_ref(&self.expr).and_then(|expr| expr.check_recursion(asg, ctx))
    }
}

impl RecursionCheck for AsgReturn {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        result_ref(&self.expr).and_then(|expr| expr.check_recursion(asg, ctx))
    }
}

impl RecursionCheck for AsgExpr {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        match self {
            Self::FnCall(expr) => expr.check_recursion(asg, ctx)?,
            Self::Literal(_) | Self::Ident(_) => {}
        }
        Ok(())
    }
}

impl RecursionCheck for AsgFnCall {
    fn check_recursion(&self, asg: &Asg, ctx: &mut FnRecursionChecker) -> crate::Result<()> {
        ctx.calls.push((self.span, self.fn_.clone()));
        self.fn_.check_recursion(asg, ctx)?;
        ctx.calls.pop();
        Ok(())
    }
}

#[derive(Default, Debug)]
struct FnRecursionChecker {
    current_fn: Option<Rc<AsgFn>>,
    calls: Vec<(Span, Rc<AsgFn>)>,
    errored_fn_indexes: FxHashSet<usize>,
    errors: Vec<SemanticError>,
}

impl FnRecursionChecker {
    fn check(&mut self, asg: &Asg) -> crate::Result<()> {
        let current_fn = self
            .current_fn
            .as_ref()
            .expect("internal error: no current function");
        if !self.is_last_call_recursive(current_fn) {
            Ok(())
        } else if self.is_error_already_generated(current_fn) {
            Err(Error)
        } else {
            for (_, fn_) in &self.calls {
                self.errored_fn_indexes.insert(fn_.index);
            }
            self.errored_fn_indexes.insert(current_fn.index);
            self.errors
                .push(fn_::recursion_found(asg, current_fn, &self.calls));
            Err(Error)
        }
    }

    fn is_last_call_recursive(&self, current_fn: &Rc<AsgFn>) -> bool {
        self.calls
            .last()
            .map_or(false, |(_, last_call)| last_call == current_fn)
    }

    fn is_error_already_generated(&self, current_fn: &Rc<AsgFn>) -> bool {
        for (_, fn_) in &self.calls {
            if self.errored_fn_indexes.contains(&fn_.index) {
                return true;
            }
        }
        self.errored_fn_indexes.contains(&current_fn.index)
    }
}

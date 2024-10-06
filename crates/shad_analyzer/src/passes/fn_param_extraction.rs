use crate::items::statement::StatementContext;
use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFn, AsgFnCall, AsgFnParam, AsgIdent, AsgIdentSource, AsgReturn,
    AsgStatement, AsgVariableDefinition,
};
use fxhash::FxHashMap;
use shad_parser::{AstExpr, AstVarDefinition};
use std::rc::Rc;

// Used to allow function parameters to be mutable.
pub(crate) fn extract_fn_params(asg: &mut Asg) {
    let fns: Vec<_> = asg.functions.values().cloned().collect();
    for fn_ in &fns {
        if !fn_.is_inlined() {
            let param_vars = param_var_defs(asg, fn_);
            let body = fn_.body_mut(asg);
            for statement in &mut body.statements {
                statement.replace_params(&param_vars);
            }
            for (index, var) in param_vars.values().enumerate() {
                body.statements
                    .insert(index, AsgStatement::Var(var.clone()));
            }
        }
    }
}

fn param_var_defs(asg: &mut Asg, fn_: &Rc<AsgFn>) -> FxHashMap<String, AsgVariableDefinition> {
    let mut ctx = StatementContext::from_scope(fn_.scope());
    fn_.params
        .iter()
        .filter(|param| param.ast.ref_span.is_none())
        .map(|param| {
            (
                param.ast.name.label.clone(),
                param_var_def(asg, &mut ctx, param),
            )
        })
        .collect()
}

fn param_var_def(
    asg: &mut Asg,
    ctx: &mut StatementContext,
    param: &Rc<AsgFnParam>,
) -> AsgVariableDefinition {
    let var_def = AstVarDefinition {
        span: param.ast.name.span,
        name: param.ast.name.clone(),
        expr: AstExpr::Ident(param.ast.name.clone()),
    };
    AsgVariableDefinition::new(asg, ctx, &var_def)
}

trait ParamVarReplacement {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>);
}

impl ParamVarReplacement for AsgStatement {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        match self {
            Self::Var(var) => var.replace_params(vars),
            Self::Assignment(assignment) => assignment.replace_params(vars),
            Self::Return(return_) => return_.replace_params(vars),
            Self::FnCall(Ok(call)) => call.replace_params(vars),
            Self::FnCall(Err(_)) => (), // no-coverage (failed analysis)
        }
    }
}

impl ParamVarReplacement for AsgVariableDefinition {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        if let Ok(expr) = &mut self.expr {
            expr.replace_params(vars);
        }
    }
}

impl ParamVarReplacement for AsgAssignment {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        if let Ok(assigned) = &mut self.assigned {
            assigned.replace_params(vars);
        }
        if let Ok(expr) = &mut self.expr {
            expr.replace_params(vars);
        }
    }
}

impl ParamVarReplacement for AsgReturn {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        if let Ok(expr) = &mut self.expr {
            expr.replace_params(vars);
        }
    }
}

impl ParamVarReplacement for AsgExpr {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        match self {
            Self::Literal(_) => (),
            Self::Ident(ident) => ident.replace_params(vars),
            Self::FnCall(call) => call.replace_params(vars),
        }
    }
}

impl ParamVarReplacement for AsgIdent {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        match &self.source {
            AsgIdentSource::Param(param) => {
                if let Some(var_def) = vars.get(&param.ast.name.label) {
                    self.source = AsgIdentSource::Var(var_def.var.clone());
                }
            }
            AsgIdentSource::Buffer(_) | AsgIdentSource::Var(_) => (),
        }
    }
}

impl ParamVarReplacement for AsgFnCall {
    fn replace_params(&mut self, vars: &FxHashMap<String, AsgVariableDefinition>) {
        for arg in &mut self.args {
            arg.replace_params(vars);
        }
    }
}

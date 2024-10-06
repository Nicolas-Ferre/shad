use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFnCall, AsgIdent, AsgIdentSource, AsgReturn, AsgStatement,
    AsgVariableDefinition,
};
use fxhash::FxHashMap;

pub(crate) trait StatementInline: Sized {
    fn inline(self, _asg: &mut Asg) -> Vec<AsgStatement> {
        unreachable!("internal error: cannot inline item")
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>);
}

impl StatementInline for AsgStatement {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        match self {
            Self::Var(var) => var.inline(asg),
            Self::Assignment(assignment) => assignment.inline(asg),
            Self::Return(return_) => return_.inline(asg),
            Self::FnCall(Ok(call)) => call.inline(asg),
            Self::FnCall(Err(_)) => unreachable!("internal error: inline invalid code"),
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        match self {
            Self::Var(var) => var.replace_params(index, param_replacements),
            Self::Assignment(assignment) => assignment.replace_params(index, param_replacements),
            Self::Return(return_) => return_.replace_params(index, param_replacements),
            Self::FnCall(Ok(call)) => call.replace_params(index, param_replacements),
            Self::FnCall(Err(_)) => unreachable!("internal error: inline invalid code"),
        }
    }
}

impl StatementInline for AsgVariableDefinition {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        if let Ok(AsgExpr::FnCall(call)) = &self.expr.as_deref() {
            if call.fn_.is_inlined() {
                let mut statements = inline_fn_statements(asg, call);
                let return_statement =
                    statement_return_expr(statements.pop().expect("internal error: no return"));
                statements.push(AsgStatement::Var(Self {
                    var: self.var,
                    expr: Ok(Box::new(return_statement)),
                }));
                statements
            } else {
                vec![AsgStatement::Var(self)]
            }
        } else {
            vec![AsgStatement::Var(self)]
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        self.var.inline_index = Some(index);
        self.expr
            .as_mut()
            .expect("internal error: inline invalid code")
            .replace_params(index, param_replacements);
    }
}

impl StatementInline for AsgAssignment {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        if let Ok(AsgExpr::FnCall(call)) = &self.expr {
            if call.fn_.is_inlined() {
                let mut statements = inline_fn_statements(asg, call);
                let return_statement =
                    statement_return_expr(statements.pop().expect("internal error: no return"));
                statements.push(AsgStatement::Assignment(Self {
                    ast: self.ast,
                    assigned: self.assigned,
                    expr: Ok(return_statement),
                    assigned_span: self.assigned_span,
                    expr_span: self.expr_span,
                }));
                statements
            } else {
                vec![AsgStatement::Assignment(self)]
            }
        } else {
            vec![AsgStatement::Assignment(self)]
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        self.assigned
            .as_mut()
            .expect("internal error: inline invalid code")
            .replace_params(index, param_replacements);
        self.expr
            .as_mut()
            .expect("internal error: inline invalid code")
            .replace_params(index, param_replacements);
    }
}

impl StatementInline for AsgReturn {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        if let Ok(AsgExpr::FnCall(call)) = &self.expr {
            if call.fn_.is_inlined() {
                let mut statements = inline_fn_statements(asg, call);
                let return_statement =
                    statement_return_expr(statements.pop().expect("internal error: no return"));
                statements.push(AsgStatement::Return(Self {
                    ast: self.ast,
                    expr: Ok(return_statement),
                }));
                statements
            } else {
                vec![AsgStatement::Return(self)]
            }
        } else {
            vec![AsgStatement::Return(self)]
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        self.expr
            .as_mut()
            .expect("internal error: inline invalid code")
            .replace_params(index, param_replacements);
    }
}

impl StatementInline for AsgExpr {
    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        match self {
            Self::Literal(_) => (),
            Self::Ident(ident) => ident.replace_params(index, param_replacements),
            Self::FnCall(call) => call.replace_params(index, param_replacements),
        }
    }
}

impl StatementInline for AsgIdent {
    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        match &mut self.source {
            AsgIdentSource::Buffer(_) => {}
            AsgIdentSource::Var(var) => var.inline_index = Some(index),
            AsgIdentSource::Param(param) => {
                self.source = param_replacements[&param.index].source.clone();
            }
        }
    }
}

impl StatementInline for AsgFnCall {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        if self.fn_.is_inlined() {
            inline_fn_statements(asg, &self)
        } else {
            vec![AsgStatement::FnCall(Ok(self))]
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        for expr in &mut self.args {
            expr.replace_params(index, param_replacements);
        }
    }
}

fn statement_return_expr(statement: AsgStatement) -> AsgExpr {
    if let AsgStatement::Return(return_) = statement {
        return_
            .expr
            .expect("internal error: expression is not inlined")
    } else {
        unreachable!("internal error: not return")
    }
}

fn inline_fn_statements(asg: &mut Asg, call: &AsgFnCall) -> Vec<AsgStatement> {
    let index = asg.next_var_index();
    let param_replacements = call
        .args
        .iter()
        .zip(&call.fn_.params)
        .map(|(arg, param)| (param.index, expr_as_ident(arg).clone()))
        .collect();
    asg.function_bodies[call.fn_.index]
        .statements
        .clone()
        .into_iter()
        .map(|mut statement| {
            statement.replace_params(index, &param_replacements);
            statement
        })
        .collect()
}

fn expr_as_ident(expr: &AsgExpr) -> &AsgIdent {
    if let AsgExpr::Ident(ident) = expr {
        ident
    } else {
        unreachable!("internal error: expression is not inlined")
    }
}

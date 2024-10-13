use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFnCall, AsgIdent, AsgIdentSource, AsgLeftValue, AsgReturn,
    AsgStatement, AsgVariableDefinition,
};
use fxhash::FxHashMap;

pub(crate) trait StatementInline: Sized {
    // coverage: off (unreachable)
    fn inline(self, _asg: &mut Asg) -> Vec<AsgStatement> {
        unreachable!("internal error: cannot inline item")
    }
    // coverage: on

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
                let (expr, mut statements) = inline_fn_expr_and_statements(asg, call);
                statements.push(AsgStatement::Var(Self {
                    var: self.var,
                    expr: Ok(Box::new(expr)),
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
        let (assigned, assigned_stmts) = if let Ok(AsgLeftValue::FnCall(call)) = &self.assigned {
            let (assigned, statements) = inline_fn_expr_and_statements(asg, call);
            (assigned.try_into(), statements)
        } else {
            (self.assigned, vec![])
        };
        let (expr, expr_stmts) = if let Ok(AsgExpr::FnCall(call)) = &self.expr {
            if call.fn_.is_inlined() {
                let (assigned, statements) = inline_fn_expr_and_statements(asg, call);
                (Ok(assigned), statements)
            } else {
                (self.expr, vec![])
            }
        } else {
            (self.expr, vec![])
        };
        [
            assigned_stmts,
            expr_stmts,
            vec![AsgStatement::Assignment(Self {
                ast: self.ast,
                assigned,
                expr,
                assigned_span: self.assigned_span,
                expr_span: self.expr_span,
            })],
        ]
        .concat()
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

impl StatementInline for AsgLeftValue {
    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        match self {
            Self::Ident(ident) => ident.replace_params(index, param_replacements),
            Self::FnCall(_) => unreachable!("internal error: left value is not inlined"),
        }
    }
}

impl StatementInline for AsgReturn {
    fn inline(self, asg: &mut Asg) -> Vec<AsgStatement> {
        if let Ok(AsgExpr::FnCall(call)) = &self.expr {
            if call.fn_.is_inlined() {
                let (expr, mut statements) = inline_fn_expr_and_statements(asg, call);
                statements.push(AsgStatement::Return(Self {
                    ast: self.ast,
                    expr: Ok(expr),
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

fn inline_fn_expr_and_statements(asg: &mut Asg, call: &AsgFnCall) -> (AsgExpr, Vec<AsgStatement>) {
    let mut statements = inline_fn_statements(asg, call);
    let return_statement =
        statement_return_expr(statements.pop().expect("internal error: no return"));
    (return_statement, statements)
}

fn inline_fn_statements(asg: &mut Asg, call: &AsgFnCall) -> Vec<AsgStatement> {
    let param_statements: Vec<_> = call.args.iter().map(|arg| inline_arg(asg, arg)).collect();
    let param_replacements = param_statements
        .iter()
        .zip(&call.fn_.params)
        .map(|((arg, _), param)| (param.index, expr_as_ident(arg).clone()))
        .collect();
    let index = asg.next_var_index();
    let param_statements = param_statements
        .into_iter()
        .flat_map(|(_, statements)| statements);
    let body_statements = asg.function_bodies[call.fn_.index]
        .statements
        .clone()
        .into_iter()
        .map(|mut statement| {
            statement.replace_params(index, &param_replacements);
            statement
        });
    param_statements.chain(body_statements).collect()
}

fn inline_arg(asg: &mut Asg, argument: &AsgExpr) -> (AsgExpr, Vec<AsgStatement>) {
    match argument {
        AsgExpr::Literal(_) | AsgExpr::Ident(_) => (argument.clone(), vec![]),
        AsgExpr::FnCall(call) => inline_fn_expr_and_statements(asg, call),
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

fn expr_as_ident(expr: &AsgExpr) -> &AsgIdent {
    if let AsgExpr::Ident(ident) = expr {
        ident
    } else {
        unreachable!("internal error: expression is not inlined")
    }
}

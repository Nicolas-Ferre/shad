use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFn, AsgFnCall, AsgIdent, AsgIdentSource, AsgReturn,
    AsgStatement, AsgVariableDefinition, FunctionListing,
};
use fxhash::FxHashMap;
use shad_parser::AstFnQualifier;
use std::mem;

pub(crate) fn inline_fns(asg: &mut Asg) {
    let mut are_functions_inlined: Vec<_> = asg
        .functions
        .values()
        .map(|fn_| fn_.ast.qualifier == AstFnQualifier::Gpu)
        .collect();
    while !are_functions_inlined.iter().all(|&is_inlined| is_inlined) {
        let fns = asg.functions.values().cloned().collect::<Vec<_>>();
        for fn_ in fns {
            if !are_functions_inlined[fn_.index]
                && are_all_dependent_fns_inlined(asg, &are_functions_inlined, &fn_)
            {
                let statements = mem::take(&mut fn_.body_mut(asg).statements);
                fn_.body_mut(asg).statements = inline(asg, statements);
                are_functions_inlined[fn_.index] = true;
            }
        }
    }
    asg.buffer_inits = mem::take(&mut asg.buffer_inits)
        .into_iter()
        .map(|statements| inline(asg, statements))
        .collect();
    asg.run_blocks = mem::take(&mut asg.run_blocks)
        .into_iter()
        .map(|statements| inline(asg, statements))
        .collect();
}

fn are_all_dependent_fns_inlined(asg: &Asg, are_fns_inlined: &[bool], fn_: &AsgFn) -> bool {
    FunctionListing::slice_functions(&asg.function_bodies[&fn_.signature].statements, asg)
        .iter()
        .all(|fn_| are_fns_inlined[fn_.index])
}

fn inline(asg: &mut Asg, statements: Vec<AsgStatement>) -> Vec<AsgStatement> {
    statements
        .into_iter()
        .flat_map(|statement| statement.split(asg).statements(Clone::clone))
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|statement| inline_statement(asg, statement))
        .collect()
}

#[derive(Debug)]
struct SplitItem<T> {
    new_statements: Vec<AsgStatement>,
    new_item: T,
}

impl<T> SplitItem<T> {
    fn new(new_statements: Vec<AsgStatement>, new_item: T) -> Self {
        Self {
            new_statements,
            new_item,
        }
    }

    fn map<U>(self, f: impl FnOnce(T) -> U) -> SplitItem<U> {
        SplitItem {
            new_statements: self.new_statements,
            new_item: f(self.new_item),
        }
    }

    fn statements(&self, item_to_statement: impl FnOnce(&T) -> AsgStatement) -> Vec<AsgStatement> {
        let mut statements = self.new_statements.clone();
        statements.push(item_to_statement(&self.new_item));
        statements
    }
}

trait StatementInline: Sized {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self>;

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>);
}

impl StatementInline for AsgStatement {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        match self {
            Self::Var(var) => var.split(asg).map(Self::Var),
            Self::Assignment(assignment) => assignment.split(asg).map(Self::Assignment),
            Self::Return(return_) => return_.split(asg).map(Self::Return),
            Self::FnCall(Ok(call)) => call.split(asg).map(|call| Self::FnCall(Ok(call))),
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
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        self.expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg)
            .map(|expr| Self {
                var: self.var.clone(),
                expr: Ok(Box::new(expr)),
            })
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
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        self.expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg)
            .map(|expr| Self {
                ast: self.ast.clone(),
                assigned: self.assigned.clone(),
                expr: Ok(expr),
                assigned_span: self.assigned_span,
                expr_span: self.expr_span,
            })
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
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        self.expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg)
            .map(|expr| Self {
                ast: self.ast.clone(),
                expr: Ok(expr),
            })
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        self.expr
            .as_mut()
            .expect("internal error: inline invalid code")
            .replace_params(index, param_replacements);
    }
}

impl StatementInline for AsgExpr {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        match self {
            Self::Literal(_) => SplitItem::new(vec![], self.clone()),
            Self::Ident(ident) => ident.split(asg).map(Self::Ident),
            Self::FnCall(call) => call.split(asg).map(Self::FnCall),
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        match self {
            Self::Literal(_) => (),
            Self::Ident(ident) => ident.replace_params(index, param_replacements),
            Self::FnCall(call) => call.replace_params(index, param_replacements),
        }
    }
}

impl StatementInline for AsgIdent {
    fn split(&self, _asg: &mut Asg) -> SplitItem<Self> {
        SplitItem::new(vec![], self.clone())
    }

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
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        if should_fn_call_be_split(self) {
            let arg_var_defs: Vec<_> = self
                .args
                .iter()
                .zip(&self.fn_.params)
                .map(|(arg, param)| {
                    param
                        .ast
                        .ref_span
                        .is_none()
                        .then(|| AsgVariableDefinition::inlined(asg, arg))
                })
                .collect();
            let new_call = Self {
                span: self.span,
                fn_: self.fn_.clone(),
                args: self
                    .args
                    .iter()
                    .zip(&arg_var_defs)
                    .map(|(arg, statement)| {
                        if let Some(statement) = statement {
                            AsgExpr::Ident(AsgIdent {
                                ast: statement.var.name.clone(),
                                source: AsgIdentSource::Var(statement.var.clone()),
                            })
                        } else {
                            arg.clone()
                        }
                    })
                    .collect(),
            };
            let statements = arg_var_defs
                .into_iter()
                .flatten()
                .map(AsgStatement::Var)
                .flat_map(|statement| statement.split(asg).statements(Clone::clone))
                .collect();
            SplitItem::new(statements, new_call)
        } else {
            SplitItem::new(vec![], self.clone())
        }
    }

    fn replace_params(&mut self, index: usize, param_replacements: &FxHashMap<usize, AsgIdent>) {
        for expr in &mut self.args {
            expr.replace_params(index, param_replacements);
        }
    }
}

fn should_fn_call_be_split(call: &AsgFnCall) -> bool {
    (call.fn_.is_inlined() && !call.args.iter().all(|arg| matches!(arg, AsgExpr::Ident(_))))
        || call.args.iter().any(should_expr_be_split)
}

fn should_expr_be_split(expr: &AsgExpr) -> bool {
    match expr {
        AsgExpr::Literal(_) | AsgExpr::Ident(_) => false,
        AsgExpr::FnCall(call) => should_fn_call_be_split(call),
    }
}

fn inline_statement(asg: &mut Asg, statement: AsgStatement) -> Vec<AsgStatement> {
    match statement {
        AsgStatement::Var(var) => {
            if let Ok(AsgExpr::FnCall(call)) = &var.expr.as_deref() {
                if call.fn_.is_inlined() {
                    let mut statements = inlined_fn_body(asg, call);
                    let return_statement =
                        statement_return_expr(statements.pop().expect("internal error: no return"));
                    statements.push(AsgStatement::Var(AsgVariableDefinition {
                        var: var.var,
                        expr: Ok(Box::new(return_statement)),
                    }));
                    statements
                } else {
                    vec![AsgStatement::Var(var)]
                }
            } else {
                vec![AsgStatement::Var(var)]
            }
        }
        AsgStatement::Assignment(assignment) => {
            if let Ok(AsgExpr::FnCall(call)) = &assignment.expr {
                if call.fn_.is_inlined() {
                    let mut statements = inlined_fn_body(asg, call);
                    let return_statement =
                        statement_return_expr(statements.pop().expect("internal error: no return"));
                    statements.push(AsgStatement::Assignment(AsgAssignment {
                        ast: assignment.ast,
                        assigned: assignment.assigned,
                        expr: Ok(return_statement),
                        assigned_span: assignment.assigned_span,
                        expr_span: assignment.expr_span,
                    }));
                    statements
                } else {
                    vec![AsgStatement::Assignment(assignment)]
                }
            } else {
                vec![AsgStatement::Assignment(assignment)]
            }
        }
        AsgStatement::Return(return_) => {
            if let Ok(AsgExpr::FnCall(call)) = &return_.expr {
                if call.fn_.is_inlined() {
                    let mut statements = inlined_fn_body(asg, call);
                    let return_statement =
                        statement_return_expr(statements.pop().expect("internal error: no return"));
                    statements.push(AsgStatement::Return(AsgReturn {
                        ast: return_.ast,
                        expr: Ok(return_statement),
                    }));
                    statements
                } else {
                    vec![AsgStatement::Return(return_)]
                }
            } else {
                vec![AsgStatement::Return(return_)]
            }
        }
        AsgStatement::FnCall(Ok(call)) => {
            if call.fn_.is_inlined() {
                inlined_fn_body(asg, &call)
            } else {
                vec![AsgStatement::FnCall(Ok(call))]
            }
        }
        AsgStatement::FnCall(Err(_)) => unreachable!("internal error: inline invalid code"),
    }
}

fn expr_as_ident(expr: &AsgExpr) -> &AsgIdent {
    if let AsgExpr::Ident(ident) = expr {
        ident
    } else {
        unreachable!("internal error: expression is not inlined")
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

fn inlined_fn_body(asg: &mut Asg, call: &AsgFnCall) -> Vec<AsgStatement> {
    let index = asg.next_var_index();
    let param_replacements = call
        .args
        .iter()
        .zip(&call.fn_.params)
        .map(|(arg, param)| (param.index, expr_as_ident(arg).clone()))
        .collect();
    asg.function_bodies[&call.fn_.signature]
        .statements
        .clone()
        .into_iter()
        .map(|mut statement| {
            statement.replace_params(index, &param_replacements);
            statement
        })
        .collect()
}

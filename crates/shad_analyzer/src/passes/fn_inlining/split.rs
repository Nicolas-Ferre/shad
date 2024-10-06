use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFnCall, AsgIdent, AsgIdentSource, AsgReturn, AsgStatement,
    AsgVariableDefinition,
};

#[derive(Debug)]
pub(crate) struct SplitItem<T> {
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

    pub(crate) fn statements(
        &self,
        item_to_statement: impl FnOnce(&T) -> AsgStatement,
    ) -> Vec<AsgStatement> {
        let mut statements = self.new_statements.clone();
        statements.push(item_to_statement(&self.new_item));
        statements
    }

    fn map<U>(self, f: impl FnOnce(T) -> U) -> SplitItem<U> {
        SplitItem {
            new_statements: self.new_statements,
            new_item: f(self.new_item),
        }
    }
}

pub(crate) trait StatementSplit: Sized {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self>;
}

impl StatementSplit for AsgStatement {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        match self {
            Self::Var(var) => var.split(asg).map(Self::Var),
            Self::Assignment(assignment) => assignment.split(asg).map(Self::Assignment),
            Self::Return(return_) => return_.split(asg).map(Self::Return),
            Self::FnCall(Ok(call)) => call.split(asg).map(|call| Self::FnCall(Ok(call))),
            Self::FnCall(Err(_)) => unreachable!("internal error: inline invalid code"),
        }
    }
}

impl StatementSplit for AsgVariableDefinition {
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
}

impl StatementSplit for AsgAssignment {
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
}

impl StatementSplit for AsgReturn {
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
}

impl StatementSplit for AsgExpr {
    fn split(&self, asg: &mut Asg) -> SplitItem<Self> {
        match self {
            Self::Literal(_) => SplitItem::new(vec![], self.clone()),
            Self::Ident(ident) => ident.split(asg).map(Self::Ident),
            Self::FnCall(call) => call.split(asg).map(Self::FnCall),
        }
    }
}

impl StatementSplit for AsgIdent {
    fn split(&self, _asg: &mut Asg) -> SplitItem<Self> {
        SplitItem::new(vec![], self.clone())
    }
}

impl StatementSplit for AsgFnCall {
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
                is_reduced: true,
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
}

fn should_fn_call_be_split(call: &AsgFnCall) -> bool {
    (call.fn_.is_inlined() && !call.is_reduced) || call.args.iter().any(should_expr_be_split)
}

fn should_expr_be_split(expr: &AsgExpr) -> bool {
    match expr {
        AsgExpr::Literal(_) | AsgExpr::Ident(_) => false,
        AsgExpr::FnCall(call) => should_fn_call_be_split(call),
    }
}

use crate::{
    Asg, AsgAssignment, AsgExpr, AsgFnCall, AsgFnParam, AsgIdent, AsgIdentSource, AsgLeftValue,
    AsgLiteral, AsgReturn, AsgStatement, AsgVariableDefinition,
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

#[derive(Default, Debug, Clone)]
pub(crate) struct StatementSplitContext {
    is_ref: bool,
    is_in_inlined_fn: bool,
}

pub(crate) trait StatementSplit: Sized {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self>;
}

impl StatementSplit for AsgStatement {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        match self {
            Self::Var(var) => var.split(asg, ctx).map(Self::Var),
            Self::Assignment(assignment) => assignment.split(asg, ctx).map(Self::Assignment),
            Self::Return(return_) => return_.split(asg, ctx).map(Self::Return),
            Self::FnCall(Ok(call)) => call.split(asg, ctx).map(|call| Self::FnCall(Ok(call))),
            Self::FnCall(Err(_)) => unreachable!("internal error: inline invalid code"),
        }
    }
}

impl StatementSplit for AsgVariableDefinition {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        self.expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg, ctx)
            .map(|expr| Self {
                var: self.var.clone(),
                expr: Ok(Box::new(expr)),
            })
    }
}

impl StatementSplit for AsgAssignment {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        let assigned_split = self
            .assigned
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg, ctx);
        let expr_split = self
            .expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg, ctx);
        SplitItem::new(
            [assigned_split.new_statements, expr_split.new_statements].concat(),
            Self {
                ast: self.ast.clone(),
                assigned: Ok(assigned_split.new_item),
                expr: Ok(expr_split.new_item),
                assigned_span: self.assigned_span,
                expr_span: self.expr_span,
            },
        )
    }
}

impl StatementSplit for AsgLeftValue {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        match self {
            Self::Ident(ident) => ident.split(asg, ctx).map(Self::Ident),
            Self::FnCall(call) => call.split(asg, ctx).map(Self::FnCall),
        }
    }
}

impl StatementSplit for AsgReturn {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        self.expr
            .as_ref()
            .expect("internal error: inline invalid code")
            .split(asg, ctx)
            .map(|expr| Self {
                ast: self.ast.clone(),
                expr: Ok(expr),
            })
    }
}

impl StatementSplit for AsgExpr {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        match self {
            Self::Literal(literal) => literal.split(asg, ctx).map(Self::Literal),
            Self::Ident(ident) => ident.split(asg, ctx).map(Self::Ident),
            Self::FnCall(call) => call.split(asg, ctx).map(Self::FnCall),
        }
    }
}

impl StatementSplit for AsgLiteral {
    fn split(&self, _asg: &mut Asg, _ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        SplitItem::new(vec![], self.clone())
    }
}

impl StatementSplit for AsgIdent {
    fn split(&self, _asg: &mut Asg, _ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        SplitItem::new(vec![], self.clone())
    }
}

impl StatementSplit for AsgFnCall {
    fn split(&self, asg: &mut Asg, ctx: &mut StatementSplitContext) -> SplitItem<Self> {
        let previous_ctx = ctx.clone();
        ctx.is_in_inlined_fn = ctx.is_in_inlined_fn || self.fn_.is_inlined();
        let split = if should_fn_call_be_split(self, ctx) {
            let splits: Vec<_> = self
                .args
                .iter()
                .zip(&self.fn_.params)
                .map(|(arg, param)| split_arg(asg, ctx, arg, param))
                .collect();
            let new_call = Self {
                span: self.span,
                fn_: self.fn_.clone(),
                args: splits.iter().map(|split| split.new_item.clone()).collect(),
            };
            let statements = splits
                .into_iter()
                .flat_map(|split| split.new_statements)
                .flat_map(|statement| {
                    statement
                        .split(asg, &mut StatementSplitContext::default())
                        .statements(Clone::clone)
                })
                .collect();
            SplitItem::new(statements, new_call)
        } else {
            SplitItem::new(vec![], self.clone())
        };
        *ctx = previous_ctx;
        split
    }
}

fn should_fn_call_be_split(call: &AsgFnCall, ctx: &mut StatementSplitContext) -> bool {
    ctx.is_in_inlined_fn
        || call.fn_.is_inlined()
        || call.args.iter().any(|arg| should_expr_be_split(arg, ctx))
}

fn should_expr_be_split(expr: &AsgExpr, ctx: &mut StatementSplitContext) -> bool {
    match expr {
        AsgExpr::Literal(_) | AsgExpr::Ident(_) => false,
        AsgExpr::FnCall(call) => should_fn_call_be_split(call, ctx),
    }
}

fn split_arg(
    asg: &mut Asg,
    ctx: &mut StatementSplitContext,
    argument: &AsgExpr,
    param: &AsgFnParam,
) -> SplitItem<AsgExpr> {
    let previous_ctx = ctx.clone();
    ctx.is_ref = param.ast.ref_span.is_some();
    let split = if ctx.is_ref {
        argument.split(asg, ctx)
    } else {
        let var_def = AsgVariableDefinition::inlined(asg, argument);
        let ident = AsgIdent {
            ast: var_def.var.name.clone(),
            source: AsgIdentSource::Var(var_def.var.clone()),
        };
        SplitItem::new(vec![AsgStatement::Var(var_def)], AsgExpr::Ident(ident))
    };
    *ctx = previous_ctx;
    split
}

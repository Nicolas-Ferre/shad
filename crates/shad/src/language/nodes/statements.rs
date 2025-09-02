use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::nodes::expressions::{BinaryOperand, TypedExpr, VarIdentExpr};
use crate::language::nodes::items::NO_RETURN_TYPE;
use crate::language::nodes::terminals::{
    EqSymbol, Ident, ReturnKeyword, SemicolonSymbol, VarKeyword,
};
use crate::language::sources;
use crate::ValidationError;

choice!(
    enum Stmt {
        LocalVarDef(LocalVarDefStmt),
        Assignment(AssignmentStmt),
        Expr(ExprStmt),
        Return(ReturnStmt),
    }
);

impl NodeConfig for Stmt {
    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        match self {
            Self::LocalVarDef(child) => child.transpile(ctx),
            Self::Assignment(child) => child.transpile(ctx),
            Self::Expr(child) => child.transpile(ctx),
            Self::Return(child) => child.transpile(ctx),
        }
    }
}

impl Stmt {
    pub(crate) fn return_(&self) -> Option<&ReturnStmt> {
        if let Self::Return(stmt) = self {
            Some(stmt)
        } else {
            None
        }
    }
}

sequence!(
    struct LocalVarDefStmt {
        var: VarKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for LocalVarDefStmt {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let id = self.id;
        let expr = self.expr.transpile(ctx);
        format!("var _{id} = {expr};")
    }
}

sequence!(
    struct AssignmentStmt {
        left: VarIdentExpr,
        eq: EqSymbol,
        #[force_error(true)]
        right: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for AssignmentStmt {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let (Some(left_type), Some(right_type)) = (
            self.left.expr_type(ctx.index),
            self.right.expr_type(ctx.index),
        ) else {
            return;
        };
        if left_type != NO_RETURN_TYPE && right_type != NO_RETURN_TYPE && left_type != right_type {
            ctx.errors.push(ValidationError::error(
                ctx,
                &*self.right,
                "invalid expression type",
                Some(&format!("expression type is `{right_type}`")),
                &[(&*self.left, &format!("expected type is `{left_type}`"))],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let left = self.left.transpile(ctx);
        let right = self.right.transpile(ctx);
        format!("{left} = {right};")
    }
}

sequence!(
    #[allow(unused_mut)]
    struct ExprStmt {
        expr: BinaryOperand,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ExprStmt {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if !self.expr.is_fn_call() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "invalid statement",
                Some("this expression must be assigned to a variable"),
                &[],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        format!("{expr};")
    }
}

sequence!(
    struct ReturnStmt {
        return_: ReturnKeyword,
        #[force_error(true)]
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ReturnStmt {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        format!("return {expr};")
    }
}

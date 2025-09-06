use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::expressions::simple::VarIdentExpr;
use crate::language::expressions::TypedExpr;
use crate::language::items::type_::NO_RETURN_TYPE;
use crate::language::keywords::{EqSymbol, RefKeyword, ReturnKeyword, SemicolonSymbol, VarKeyword};
use crate::language::patterns::Ident;
use crate::language::sources;
use crate::ValidationError;

choice!(
    enum Stmt {
        LocalVarDef(LocalVarDefStmt),
        LocalRefDef(LocalRefDefStmt),
        Assignment(AssignmentStmt),
        Expr(ExprStmt),
        Return(ReturnStmt),
    }
);

impl NodeConfig for Stmt {
    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        match self {
            Self::LocalVarDef(child) => child.transpile(ctx),
            Self::LocalRefDef(child) => child.transpile(ctx),
            Self::Assignment(child) => child.transpile(ctx),
            Self::Expr(child) => child.transpile(ctx),
            Self::Return(child) => child.transpile(ctx),
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

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let var_name = if ctx.inline_state.is_inlined {
            let id = ctx.next_node_id();
            let var_name = format!("_{id}");
            ctx.add_inline_mapping(self.id, &var_name);
            var_name
        } else {
            format!("_{}", self.id)
        };
        let expr = self.expr.transpile(ctx);
        format!("var {var_name} = {expr};")
    }
}

sequence!(
    struct LocalRefDefStmt {
        ref_: RefKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for LocalRefDefStmt {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.expr.expr_type(index)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        if self.expr.is_ref(ctx.index) {
            ctx.add_inline_mapping(self.id, expr);
            String::new()
        } else {
            let id = ctx.next_node_id();
            let var_name = format!("_{id}");
            ctx.add_inline_mapping(self.id, &var_name);
            format!("var {var_name} = {expr};")
        }
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
        expr: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ExprStmt {
    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        if self.expr.expr_type(ctx.index).as_deref() == Some(NO_RETURN_TYPE) {
            format!("{expr};")
        } else {
            let id = ctx.next_node_id();
            format!("var _{id} = {expr};")
        }
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
        if ctx.inline_state.is_returning_ref {
            if self.expr.is_ref(ctx.index) {
                ctx.inline_state.returned_ref = Some(expr);
                String::new()
            } else {
                let id = ctx.next_node_id();
                let var_name = format!("_{id}");
                ctx.inline_state.returned_ref = Some(var_name.clone());
                format!("var {var_name} = {expr};")
            }
        } else if let Some(var_id) = ctx.inline_state.return_var_id {
            format!("_{var_id} = {expr};")
        } else {
            format!("return {expr};")
        }
    }
}

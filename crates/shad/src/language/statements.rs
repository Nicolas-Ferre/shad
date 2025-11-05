use crate::compilation::constant::{ConstantContext, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, Node, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::keywords::{EqSymbol, RefKeyword, ReturnKeyword, SemicolonSymbol, VarKeyword};
use crate::language::patterns::Ident;
use crate::language::{sources, validations};
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

sequence!(
    struct LocalVarDefStmt {
        var: VarKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for LocalVarDefStmt {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_no_return_type(&*self.expr, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let value = self.expr.evaluate_constant(ctx)?;
        ctx.create_var(self.id, value);
        None
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
        expr: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for LocalRefDefStmt {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_no_return_type(&*self.expr, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let value = self.expr.evaluate_constant(ctx)?;
        ctx.create_var(self.id, value);
        None
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        if self.expr.is_ref(ctx.index) == Some(true) {
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
        left: MaybeBinaryExpr,
        eq: EqSymbol,
        #[force_error(true)]
        right: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for AssignmentStmt {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.left.is_ref(ctx.index) == Some(false) {
            ctx.errors.push(ValidationError::error(
                ctx,
                &*self.left,
                "invalid assignment left value",
                Some("this should be a valid reference"),
                &[],
            ));
        }
        validations::check_invalid_expr_type(&*self.left, &*self.right, false, ctx);
        validations::check_no_return_type(&*self.right, ctx);
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        Some(self)
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
    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        Some(self)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        if self
            .expr
            .type_(ctx.index)
            .is_some_and(NodeType::is_no_return)
        {
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
        expr: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ReturnStmt {
    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.expr.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_no_return_type(&*self.expr, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.expr.evaluate_constant(ctx)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        if ctx.inline_state.is_returning_ref {
            if self.expr.is_ref(ctx.index) == Some(true) {
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

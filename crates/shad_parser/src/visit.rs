use crate::fn_call::AstFnCall;
use crate::{
    Ast, AstAssignment, AstBufferItem, AstExpr, AstFnCallStatement, AstFnItem, AstIdent, AstItem,
    AstLeftValue, AstLiteral, AstReturn, AstRunItem, AstStatement, AstVarDefinition,
};

// coverage: off (not all functions are used by other crates)
macro_rules! visit_trait {
    ($name:ident: $($mut_keyword:tt)?) => {
        /// A trait for visiting an AST.
        #[allow(unused_variables)]
        pub trait $name {
            /// Runs logic when entering in an AST.
            fn enter_ast(&mut self, node: &$($mut_keyword)? Ast) {}

            /// Runs logic when entering in an item.
            fn enter_item(&mut self, node: &$($mut_keyword)? AstItem) {}

            /// Runs logic when entering in a buffer item.
            fn enter_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {}

            /// Runs logic when entering in a function item.
            fn enter_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {}

            /// Runs logic when entering in a statement.
            fn enter_statement(&mut self, node: &$($mut_keyword)? AstStatement) {}

            /// Runs logic when entering in a `run` item.
            fn enter_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {}

            /// Runs logic when entering in an assignment.
            fn enter_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {}

            /// Runs logic when entering in a left value.
            fn enter_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {}

            /// Runs logic when entering in a variable definition.
            fn enter_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {}

            /// Runs logic when entering in an `return` statement.
            fn enter_return(&mut self, node: &$($mut_keyword)? AstReturn) {}

            /// Runs logic when entering in a function call statement.
            fn enter_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {}

            /// Runs logic when entering in an expression.
            fn enter_expr(&mut self, node: &$($mut_keyword)? AstExpr) {}

            /// Runs logic when entering in a function call.
            fn enter_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {}

            /// Runs logic when entering in a literal.
            fn enter_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {}

            /// Runs logic when entering in an identifier.
            fn enter_ident(&mut self, node: &$($mut_keyword)? AstIdent) {}

            /// Runs logic when exiting an AST.
            fn exit_ast(&mut self, node: &$($mut_keyword)? Ast) {}

            /// Runs logic when exiting an item.
            fn exit_item(&mut self, node: &$($mut_keyword)? AstItem) {}

            /// Runs logic when exiting a buffer item.
            fn exit_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {}

            /// Runs logic when exiting a function item.
            fn exit_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {}

            /// Runs logic when exiting a statement.
            fn exit_statement(&mut self, node: &$($mut_keyword)? AstStatement) {}

            /// Runs logic when exiting a `run` item.
            fn exit_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {}

            /// Runs logic when exiting an assignment.
            fn exit_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {}

            /// Runs logic when exiting a left value.
            fn exit_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {}

            /// Runs logic when exiting a variable definition.
            fn exit_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {}

            /// Runs logic when exiting a `return` statement.
            fn exit_return(&mut self, node: &$($mut_keyword)? AstReturn) {}

            /// Runs logic when exiting a function call statement.
            fn exit_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {}

            /// Runs logic when exiting an expression.
            fn exit_expr(&mut self, node: &$($mut_keyword)? AstExpr) {}

            /// Runs logic when exiting a function call.
            fn exit_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {}

            /// Runs logic when exiting a literal.
            fn exit_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {}

            /// Runs logic when exiting an identifier.
            fn exit_ident(&mut self, node: &$($mut_keyword)? AstIdent) {}

            /// Visit an AST.
            fn visit_ast(&mut self, node: &$($mut_keyword)? Ast) {
                self.enter_ast(node);
                for node in &$($mut_keyword)? node.items {
                    self.visit_item(node);
                }
                self.exit_ast(node);
            }

            /// Visit an item.
            fn visit_item(&mut self, node: &$($mut_keyword)? AstItem) {
                self.enter_item(node);
                match node {
                    AstItem::Buffer(node) => self.visit_buffer_item(node),
                    AstItem::Fn(node) => self.visit_fn_item(node),
                    AstItem::Run(node) => self.visit_run_item(node),
                }
                self.exit_item(node);
            }

            /// Visit a buffer item.
            fn visit_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {
                self.enter_buffer_item(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                self.visit_expr(&$($mut_keyword)? node.value);
                self.exit_buffer_item(node);
            }

            /// Visit a function item.
            fn visit_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {
                self.enter_fn_item(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                for node in &$($mut_keyword)? node.statements {
                    self.visit_statement(node);
                }
                self.exit_fn_item(node);
            }

            /// Visit a statement.
            fn visit_statement(&mut self, node: &$($mut_keyword)? AstStatement) {
                self.enter_statement(node);
                match node {
                    AstStatement::Assignment(node) => self.visit_assignment(node),
                    AstStatement::Var(node) => self.visit_var_definition(node),
                    AstStatement::Return(node) => self.visit_return(node),
                    AstStatement::FnCall(node) => self.visit_fn_call_statement(node),
                }
                self.exit_statement(node);
            }

            /// Visit a `run` item.
            fn visit_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {
                self.enter_run_item(node);
                for node in &$($mut_keyword)? node.statements {
                    self.visit_statement(node);
                }
                self.exit_run_item(node);
            }

            /// Visit an assignment.
            fn visit_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {
                self.enter_assignment(node);
                self.visit_left_value(&$($mut_keyword)? node.value);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_assignment(node);
            }

            /// Visit a left value.
            fn visit_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {
                self.enter_left_value(node);
                match node {
                    AstLeftValue::Ident(node) => self.visit_ident(node),
                    AstLeftValue::FnCall(node) => self.visit_fn_call(node),
                }
                self.exit_left_value(node);
            }

            /// Visit a variable definition.
            fn visit_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {
                self.enter_var_definition(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_var_definition(node);
            }

            /// Visit a `return` statement.
            fn visit_return(&mut self, node: &$($mut_keyword)? AstReturn) {
                self.enter_return(node);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_return(node);
            }

            /// Visit a function call statement.
            fn visit_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {
                self.enter_fn_call_statement(node);
                self.visit_fn_call(&$($mut_keyword)? node.call);
                self.exit_fn_call_statement(node);
            }

            /// Visit an expression.
            fn visit_expr(&mut self, node: &$($mut_keyword)? AstExpr) {
                self.enter_expr(node);
                match node {
                    AstExpr::Literal(node) => self.visit_literal(node),
                    AstExpr::Ident(node) => self.visit_ident(node),
                    AstExpr::FnCall(node) => self.visit_fn_call(node),
                }
                self.exit_expr(node);
            }

            /// Visit a function call.
            fn visit_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {
                self.enter_fn_call(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                for node in &$($mut_keyword)? node.args {
                    self.visit_expr(node);
                }
                self.exit_fn_call(node);
            }

            /// Visit a literal.
            fn visit_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {
                self.enter_literal(node);
                self.exit_literal(node);
            }

            /// Visit an identifier.
            fn visit_ident(&mut self, node: &$($mut_keyword)? AstIdent) {
                self.enter_ident(node);
                self.exit_ident(node);
            }
        }
    };
}
// coverage: on

visit_trait!(Visit: );
visit_trait!(VisitMut: mut);

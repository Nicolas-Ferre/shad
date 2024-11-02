#![allow(missing_docs)] // TODO: remove

use crate::fn_call::AstFnCall;
use crate::{
    Ast, AstAssignment, AstBufferItem, AstExpr, AstFnCallStatement, AstFnItem, AstIdent, AstItem,
    AstLeftValue, AstLiteral, AstReturn, AstRunItem, AstStatement, AstVarDefinition,
};

macro_rules! visit_trait {
    ($name:ident: $($mut_keyword:tt)?) => {
        #[allow(unused_variables)]
        pub trait $name {
            fn enter_ast(&mut self, node: &$($mut_keyword)? Ast) {}

            fn enter_item(&mut self, node: &$($mut_keyword)? AstItem) {}

            fn enter_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {}

            fn enter_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {}

            fn enter_statement(&mut self, node: &$($mut_keyword)? AstStatement) {}

            fn enter_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {}

            fn enter_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {}

            fn enter_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {}

            fn enter_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {}

            fn enter_return(&mut self, node: &$($mut_keyword)? AstReturn) {}

            fn enter_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {}

            fn enter_expr(&mut self, node: &$($mut_keyword)? AstExpr) {}

            fn enter_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {}

            fn enter_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {}

            fn enter_ident(&mut self, node: &$($mut_keyword)? AstIdent) {}

            fn exit_ast(&mut self, node: &$($mut_keyword)? Ast) {}

            fn exit_item(&mut self, node: &$($mut_keyword)? AstItem) {}

            fn exit_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {}

            fn exit_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {}

            fn exit_statement(&mut self, node: &$($mut_keyword)? AstStatement) {}

            fn exit_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {}

            fn exit_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {}

            fn exit_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {}

            fn exit_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {}

            fn exit_return(&mut self, node: &$($mut_keyword)? AstReturn) {}

            fn exit_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {}

            fn exit_expr(&mut self, node: &$($mut_keyword)? AstExpr) {}

            fn exit_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {}

            fn exit_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {}

            fn exit_ident(&mut self, node: &$($mut_keyword)? AstIdent) {}

            fn visit_ast(&mut self, node: &$($mut_keyword)? Ast) {
                self.enter_ast(node);
                for node in &$($mut_keyword)? node.items {
                    self.visit_item(node);
                }
                self.exit_ast(node);
            }

            fn visit_item(&mut self, node: &$($mut_keyword)? AstItem) {
                self.enter_item(node);
                match node {
                    AstItem::Buffer(node) => self.visit_buffer_item(node),
                    AstItem::Fn(node) => self.visit_fn_item(node),
                    AstItem::Run(node) => self.visit_run_item(node),
                }
                self.exit_item(node);
            }

            fn visit_buffer_item(&mut self, node: &$($mut_keyword)? AstBufferItem) {
                self.enter_buffer_item(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                self.visit_expr(&$($mut_keyword)? node.value);
                self.exit_buffer_item(node);
            }

            fn visit_fn_item(&mut self, node: &$($mut_keyword)? AstFnItem) {
                self.enter_fn_item(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                for node in &$($mut_keyword)? node.statements {
                    self.visit_statement(node);
                }
                self.exit_fn_item(node);
            }

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

            fn visit_run_item(&mut self, node: &$($mut_keyword)? AstRunItem) {
                self.enter_run_item(node);
                for node in &$($mut_keyword)? node.statements {
                    self.visit_statement(node);
                }
                self.exit_run_item(node);
            }

            fn visit_assignment(&mut self, node: &$($mut_keyword)? AstAssignment) {
                self.enter_assignment(node);
                self.visit_left_value(&$($mut_keyword)? node.value);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_assignment(node);
            }

            fn visit_left_value(&mut self, node: &$($mut_keyword)? AstLeftValue) {
                self.enter_left_value(node);
                match node {
                    AstLeftValue::Ident(node) => self.visit_ident(node),
                    AstLeftValue::FnCall(node) => self.visit_fn_call(node),
                }
                self.exit_left_value(node);
            }

            fn visit_var_definition(&mut self, node: &$($mut_keyword)? AstVarDefinition) {
                self.enter_var_definition(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_var_definition(node);
            }

            fn visit_return(&mut self, node: &$($mut_keyword)? AstReturn) {
                self.enter_return(node);
                self.visit_expr(&$($mut_keyword)? node.expr);
                self.exit_return(node);
            }

            fn visit_fn_call_statement(&mut self, node: &$($mut_keyword)? AstFnCallStatement) {
                self.enter_fn_call_statement(node);
                self.visit_fn_call(&$($mut_keyword)? node.call);
                self.exit_fn_call_statement(node);
            }

            fn visit_expr(&mut self, node: &$($mut_keyword)? AstExpr) {
                self.enter_expr(node);
                match node {
                    AstExpr::Literal(node) => self.visit_literal(node),
                    AstExpr::Ident(node) => self.visit_ident(node),
                    AstExpr::FnCall(node) => self.visit_fn_call(node),
                }
                self.exit_expr(node);
            }

            fn visit_fn_call(&mut self, node: &$($mut_keyword)? AstFnCall) {
                self.enter_fn_call(node);
                self.visit_ident(&$($mut_keyword)? node.name);
                for node in &$($mut_keyword)? node.args {
                    self.visit_expr(node);
                }
                self.exit_fn_call(node);
            }

            fn visit_literal(&mut self, node: &$($mut_keyword)? AstLiteral) {
                self.enter_literal(node);
                self.exit_literal(node);
            }

            fn visit_ident(&mut self, node: &$($mut_keyword)? AstIdent) {
                self.enter_ident(node);
                self.exit_ident(node);
            }
        }
    };
}

visit_trait!(Visit: );
visit_trait!(VisitMut: mut);

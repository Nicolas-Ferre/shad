use crate::resolver::ExprSemantic;
use crate::{errors, resolver, Analysis, Function, NO_RETURN_TYPE};
use shad_error::SemanticError;
use shad_parser::{
    AstAssignment, AstExpr, AstFnCall, AstFnItem, AstReturn, AstStatement, AstVarDefinition, Visit,
};

pub(crate) fn check(analysis: &mut Analysis) {
    let mut checker = StatementCheck::new(analysis);
    for block in &analysis.init_blocks {
        checker.module = &block.buffer.module;
        checker.visit_run_item(&block.ast);
    }
    for block in &analysis.run_blocks {
        checker.module = &block.module;
        checker.visit_run_item(&block.ast);
    }
    for fn_ in analysis.fns.values() {
        checker.module = &fn_.ast.name.span.module.name;
        checker.fn_ = Some(fn_);
        checker.visit_fn_item(&fn_.ast);
    }
    analysis.errors.extend(checker.errors);
}

struct StatementCheck<'a> {
    analysis: &'a Analysis,
    errors: Vec<SemanticError>,
    fn_: Option<&'a Function>,
    module: &'a str,
}

impl<'a> StatementCheck<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            errors: vec![],
            fn_: None,
            module: "",
        }
    }

    fn check_invalid_expr_type(&mut self, expr: &AstExpr) {
        if let Some(type_id) = resolver::expr_type(self.analysis, expr) {
            if type_id.name == NO_RETURN_TYPE {
                let error = errors::expressions::invalid_type(expr, &type_id);
                self.errors.push(error);
            }
        }
    }
}

impl Visit for StatementCheck<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        let fn_ =
            resolver::fn_(self.analysis, &node.name).expect("internal error: missing function");
        if let Some(return_pos) = node
            .statements
            .iter()
            .position(|statement| matches!(statement, AstStatement::Return(_)))
        {
            if return_pos + 1 < node.statements.len() {
                self.errors.push(errors::returns::statement_after(
                    &node.statements[return_pos],
                    &node.statements[return_pos + 1],
                ));
            }
        } else if node.return_type.is_some() && !node.is_gpu {
            let error = errors::returns::missing_return(node, &fn_.id);
            self.errors.push(error);
        }
    }

    fn enter_assignment(&mut self, node: &AstAssignment) {
        self.check_invalid_expr_type(&node.right);
        let expected_type = resolver::expr_type(self.analysis, &node.left);
        let expr_type = resolver::expr_type(self.analysis, &node.right);
        if let (Some(expected_type), Some(expr_type)) = (expected_type, expr_type) {
            if expected_type != expr_type {
                self.errors.push(errors::assignments::invalid_type(
                    node,
                    &expected_type,
                    &expr_type,
                ));
            }
        }
        if resolver::expr_semantic(self.analysis, &node.left) == ExprSemantic::Value {
            self.errors.push(errors::expressions::not_ref(&node.left));
        }
    }

    fn enter_var_definition(&mut self, node: &AstVarDefinition) {
        self.check_invalid_expr_type(&node.expr);
        let semantic = resolver::expr_semantic(self.analysis, &node.expr);
        if node.is_ref && semantic == ExprSemantic::Value {
            self.errors.push(errors::expressions::not_ref(&node.expr));
        }
    }

    fn enter_return(&mut self, node: &AstReturn) {
        if let Some(fn_) = self.fn_ {
            if let Some(return_type) = &fn_.ast.return_type {
                let Some(type_id) = resolver::expr_type(self.analysis, &node.expr) else {
                    return;
                };
                if let Some(return_type_id) = &fn_.return_type_id {
                    if &type_id != return_type_id {
                        self.errors.push(errors::returns::invalid_type(
                            node,
                            &fn_.ast,
                            &type_id,
                            return_type_id,
                        ));
                        return;
                    }
                }
                let semantic = resolver::expr_semantic(self.analysis, &node.expr);
                if return_type.is_ref && semantic == ExprSemantic::Value {
                    self.errors.push(errors::expressions::not_ref(&node.expr));
                }
            } else {
                self.errors.push(errors::returns::no_return_type(node));
            }
        } else {
            self.errors.push(errors::returns::outside_fn(node));
        }
    }

    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolver::fn_(self.analysis, &node.name) {
            for (arg, param) in node.args.iter().zip(&fn_.ast.params) {
                if let Some(name) = &arg.name {
                    if param.name.label != name.label {
                        let error = errors::fn_calls::invalid_param_name(name, &param.name);
                        self.errors.push(error);
                    }
                }
            }
        }
    }
}

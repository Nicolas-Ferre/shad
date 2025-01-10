use crate::resolving::expressions::ExprSemantic;
use crate::{errors, resolving, Analysis, FnId, Function, NO_RETURN_TYPE};
use shad_error::SemanticError;
use shad_parser::{
    AstAssignment, AstExpr, AstExprRoot, AstFnCall, AstFnItem, AstReturn, AstStatement,
    AstVarDefinition, Visit,
};

pub(crate) fn check(analysis: &mut Analysis) {
    let mut checker = StatementCheck::new(analysis, false);
    for block in &analysis.init_blocks {
        checker.module = &block.buffer.module;
        checker.visit_run_item(&block.ast);
    }
    for block in &analysis.run_blocks {
        checker.module = &block.module;
        checker.visit_run_item(&block.ast);
    }
    for constant in analysis.constants.values() {
        checker.module = &constant.id.module;
        checker.is_const_context = true;
        checker.visit_expr(&constant.ast.value);
    }
    for fn_ in analysis.fns.values() {
        checker.module = &fn_.ast.name.span.module.name;
        checker.fn_ = Some(fn_);
        checker.is_const_context = false;
        checker.visit_fn_item(&fn_.ast);
    }
    analysis.errors.extend(checker.errors);
}

struct StatementCheck<'a> {
    analysis: &'a Analysis,
    errors: Vec<SemanticError>,
    fn_: Option<&'a Function>,
    module: &'a str,
    is_const_context: bool,
}

impl<'a> StatementCheck<'a> {
    fn new(analysis: &'a Analysis, is_const_context: bool) -> Self {
        Self {
            analysis,
            errors: vec![],
            fn_: None,
            module: "",
            is_const_context,
        }
    }

    fn check_invalid_expr_type(&mut self, expr: &AstExpr) {
        if let Some(type_id) = resolving::types::expr(self.analysis, expr) {
            if type_id == NO_RETURN_TYPE {
                let type_ = &self.analysis.types[&type_id];
                let error = errors::expressions::invalid_type(expr, type_);
                self.errors.push(error);
            }
        }
    }
}

impl Visit for StatementCheck<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        let fn_ = &self.analysis.fns[&FnId::from_item(self.analysis, node)];
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
        } else if node.return_type.is_some() && node.gpu_qualifier.is_none() {
            let error = errors::returns::missing_return(node, &fn_.id.signature(self.analysis));
            self.errors.push(error);
        }
    }

    fn enter_assignment(&mut self, node: &AstAssignment) {
        self.check_invalid_expr_type(&node.right);
        let expected_type_id = resolving::types::expr(self.analysis, &node.left);
        let expr_type_id = resolving::types::expr(self.analysis, &node.right);
        if let (Some(expected_type_id), Some(expr_type_id)) = (expected_type_id, expr_type_id) {
            if expected_type_id != expr_type_id {
                self.errors.push(errors::assignments::invalid_type(
                    node,
                    &self.analysis.types[&expected_type_id],
                    &self.analysis.types[&expr_type_id],
                ));
            }
        }
        if resolving::expressions::semantic(self.analysis, &node.left) == ExprSemantic::Value {
            self.errors.push(errors::expressions::not_ref(&node.left));
        }
    }

    fn enter_var_definition(&mut self, node: &AstVarDefinition) {
        self.check_invalid_expr_type(&node.expr);
        let semantic = resolving::expressions::semantic(self.analysis, &node.expr);
        if node.is_ref && semantic == ExprSemantic::Value {
            self.errors.push(errors::expressions::not_ref(&node.expr));
        }
    }

    fn enter_return(&mut self, node: &AstReturn) {
        if let Some(fn_) = self.fn_ {
            if let Some(return_type) = &fn_.ast.return_type {
                let Some(type_id) = resolving::types::expr(self.analysis, &node.expr) else {
                    return;
                };
                if let Some(return_type_id) = &fn_.return_type_id {
                    if &type_id != return_type_id {
                        self.errors.push(errors::returns::invalid_type(
                            node,
                            &fn_.ast,
                            &self.analysis.types[&type_id],
                            &self.analysis.types[return_type_id],
                        ));
                        return;
                    }
                }
                let semantic = resolving::expressions::semantic(self.analysis, &node.expr);
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
        if let Some(fn_) = resolving::items::fn_(self.analysis, node) {
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

    fn enter_expr(&mut self, node: &AstExpr) {
        match &node.root {
            AstExprRoot::Ident(ident) => {
                if (!self.is_const_context && self.analysis.item(ident).is_none())
                    || (self.is_const_context
                        && resolving::items::constant(self.analysis, ident).is_none())
                {
                    let error = errors::variables::not_found(ident);
                    self.errors.push(error);
                }
            }
            AstExprRoot::FnCall(call) => {
                if resolving::items::fn_(self.analysis, call).is_none() {
                    if let Some(arg_type_ids) = resolving::types::fn_args(self.analysis, call) {
                        if self.analysis.fn_(call).is_none() {
                            let error = errors::functions::not_found(
                                call,
                                arg_type_ids
                                    .iter()
                                    .map(|type_id| &self.analysis.types[type_id]),
                            );
                            self.errors.push(error);
                        }
                    }
                }
            }
            AstExprRoot::Literal(_) => (),
        }
        let mut last_type_id = resolving::types::expr_root(self.analysis, node);
        for field in &node.fields {
            if let Some(type_id) = &last_type_id {
                let type_field = resolving::items::field(self.analysis, type_id, field);
                if let Some(type_field) = type_field {
                    last_type_id.clone_from(&type_field.type_id);
                } else {
                    let type_ = &self.analysis.types[type_id];
                    let error = errors::types::field_not_found(field, type_);
                    self.errors.push(error);
                    return;
                }
            } else {
                return;
            }
        }
    }
}

use crate::registration::types;
use crate::{errors, Analysis, Function};
use shad_error::SemanticError;
use shad_parser::{
    AstAssignment, AstExpr, AstFnCall, AstFnItem, AstFnQualifier, AstIdentType, AstLeftValue,
    AstReturn, AstStatement, AstVarDefinition, Visit,
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

    fn expr_semantic(&self, expr: &AstExpr) -> ExprSemantic {
        match expr {
            AstExpr::Literal(_) => ExprSemantic::Value,
            AstExpr::Ident(_) => ExprSemantic::Ref,
            AstExpr::FnCall(call) => self
                .analysis
                .fn_id(&call.name)
                .and_then(|id| self.analysis.fns[&id].ast.return_type.as_ref())
                .map_or(ExprSemantic::None, |type_| {
                    if type_.is_ref {
                        ExprSemantic::Ref
                    } else {
                        ExprSemantic::Value
                    }
                }),
        }
    }
}

impl Visit for StatementCheck<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        let fn_id = self
            .analysis
            .fn_id(&node.name)
            .expect("internal error: missing function");
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
        } else if node.return_type.is_some() && node.qualifier != AstFnQualifier::Gpu {
            self.errors
                .push(errors::returns::missing_return(node, &fn_id));
        }
    }

    fn enter_assignment(&mut self, node: &AstAssignment) {
        let value_id = match &node.value {
            AstLeftValue::Ident(ident) => ident.id,
            AstLeftValue::FnCall(call) => call.name.id,
        };
        let expected_type = self
            .analysis
            .idents
            .get(&value_id)
            .and_then(|ident| ident.type_.as_ref());
        let expr_type = self.analysis.expr_type(&node.expr);
        if let (Some(expected_type), Some(expr_type)) = (expected_type, expr_type) {
            if expected_type != &expr_type {
                self.errors.push(errors::assignments::invalid_type(
                    node,
                    expected_type,
                    &expr_type,
                ));
            }
        }
    }

    fn enter_left_value(&mut self, node: &AstLeftValue) {
        let expr = node.clone().into();
        if self.expr_semantic(&expr) == ExprSemantic::Value {
            self.errors.push(errors::expressions::not_ref(&expr));
        }
    }

    fn enter_var_definition(&mut self, node: &AstVarDefinition) {
        if node.name.type_ == AstIdentType::RefDef
            && self.expr_semantic(&node.expr) == ExprSemantic::Value
        {
            self.errors.push(errors::expressions::not_ref(&node.expr));
        }
    }

    fn enter_return(&mut self, node: &AstReturn) {
        if let Some(fn_) = self.fn_ {
            if let Some(return_type) = &fn_.ast.return_type {
                let Some(type_id) = self.analysis.expr_type(&node.expr) else {
                    return;
                };
                if let Some(return_type_id) =
                    types::find(self.analysis, self.module, &return_type.name)
                {
                    if type_id != return_type_id {
                        self.errors.push(errors::returns::invalid_type(
                            node,
                            &fn_.ast,
                            &type_id,
                            &return_type_id,
                        ));
                        return;
                    }
                }
                if return_type.is_ref && self.expr_semantic(&node.expr) == ExprSemantic::Value {
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
        if let Some(fn_id) = self.analysis.fn_id(&node.name) {
            let fn_ = &self.analysis.fns[&fn_id];
            for (arg, param) in node.args.iter().zip(&fn_.ast.params) {
                if let (Some(ref_span), ExprSemantic::Value) =
                    (param.ref_span.clone(), self.expr_semantic(arg))
                {
                    self.errors
                        .push(errors::fn_calls::invalid_ref(arg, ref_span));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprSemantic {
    None,
    Ref,
    Value,
}

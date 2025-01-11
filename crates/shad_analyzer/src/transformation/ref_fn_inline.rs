use crate::{listing, registration, resolving, Analysis, FnId};
use fxhash::FxHashMap;
use shad_parser::{
    AstExpr, AstExprRoot, AstExprStatement, AstFnCall, AstFnItem, AstLiteral, AstLiteralType,
    AstStatement, VisitMut,
};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    transform_fns(analysis);
    super::transform_init_blocks(analysis, visit_statements);
    super::transform_run_blocks(analysis, visit_statements);
}

fn transform_fns(analysis: &mut Analysis) {
    let mut are_fns_inlined: FxHashMap<_, _> = analysis
        .fns
        .iter()
        .map(|(fn_id, fn_)| (fn_id.clone(), fn_.ast.gpu_qualifier.is_some()))
        .collect();
    let ids: Vec<_> = analysis.fns.keys().cloned().collect();
    while are_fns_inlined.values().any(|is_inlined| !is_inlined) {
        for id in &ids {
            if !are_fns_inlined[id] && are_all_dependent_fns_inlined(analysis, &are_fns_inlined, id)
            {
                let mut fn_ = analysis.fns[id].clone();
                visit_statements(analysis, &mut fn_.ast.statements);
                are_fns_inlined.insert(id.clone(), true);
                analysis.fns.insert(id.clone(), fn_);
            }
        }
    }
}

fn are_all_dependent_fns_inlined(
    analysis: &Analysis,
    are_fns_inlined: &FxHashMap<FnId, bool>,
    fn_id: &FnId,
) -> bool {
    analysis.fns[fn_id]
        .ast
        .statements
        .iter()
        .flat_map(|s| listing::functions::list_in_statement(analysis, s))
        .all(|id| are_fns_inlined[&id])
}

fn visit_statements(analysis: &mut Analysis, statements: &mut Vec<AstStatement>) {
    *statements = mem::take(statements)
        .into_iter()
        .flat_map(|mut statement| {
            let mut transform = RefFnInlineTransform::new(analysis);
            transform.visit_statement(&mut statement);
            transform.statements.push(statement);
            transform.statements
        })
        .collect();
}

struct RefFnInlineTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> RefFnInlineTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }
}

impl VisitMut for RefFnInlineTransform<'_> {
    fn exit_expr_statement(&mut self, node: &mut AstExprStatement) {
        if let AstExprRoot::FnCall(call) = &node.expr.root {
            if let Some(fn_) = resolving::items::fn_(self.analysis, call) {
                if fn_.is_inlined {
                    node.expr = AstLiteral {
                        span: node.span.clone(),
                        raw_value: "0".to_string(),
                        cleaned_value: "0".to_string(),
                        type_: AstLiteralType::I32,
                    }
                    .into();
                }
            } else {
                unreachable!("internal error: missing function");
            }
        }
    }

    fn exit_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::FnCall(call) = &node.root {
            let statements = inlined_fn_statements(self.analysis, call);
            if !statements.is_empty() {
                self.statements.extend(statements);
                let last_statement = self
                    .statements
                    .pop()
                    .expect("internal error: missing return");
                if let AstStatement::Return(return_) = last_statement {
                    node.replace_root(return_.expr);
                } else {
                    self.statements.push(last_statement);
                }
            }
        }
    }
}

fn inlined_fn_statements(analysis: &mut Analysis, call: &AstFnCall) -> Vec<AstStatement> {
    if let Some(fn_) = resolving::items::fn_(analysis, call) {
        let mut fn_ = fn_.clone();
        if fn_.is_inlined {
            registration::vars::register_fn(analysis, &mut fn_);
            let mut transform = RefFnStatementsTransform::new(&fn_.ast, call);
            fn_.ast
                .statements
                .into_iter()
                .map(|mut statement| {
                    transform.visit_statement(&mut statement);
                    statement
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        unreachable!("internal error: missing function");
    }
}

struct RefFnStatementsTransform {
    param_args: FxHashMap<String, AstExpr>,
}

impl RefFnStatementsTransform {
    fn new(fn_: &AstFnItem, call: &AstFnCall) -> Self {
        Self {
            param_args: fn_
                .params
                .iter()
                .zip(&call.args)
                .map(|(param, arg)| (param.name.label.clone(), arg.value.clone()))
                .collect(),
        }
    }
}

impl VisitMut for RefFnStatementsTransform {
    fn enter_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let Some(new_root) = self.param_args.get(&ident.label) {
                node.replace_root(new_root.clone());
            }
        }
    }
}

use crate::{listing, resolver, Analysis, FnId, Ident, IdentSource};
use fxhash::FxHashMap;
use shad_parser::{
    AstExpr, AstExprStatement, AstFnCall, AstFnItem, AstFnQualifier, AstIdent, AstLiteral,
    AstLiteralType, AstStatement, AstValue, AstValueRoot, VisitMut,
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
        .map(|(fn_id, fn_)| (fn_id.clone(), fn_.ast.qualifier == AstFnQualifier::Gpu))
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
        if let AstExpr::Value(value) = &node.expr {
            if let AstValueRoot::FnCall(call) = &value.root {
                if resolver::fn_(self.analysis, &call.name)
                    .expect("internal error: missing function")
                    .is_inlined
                {
                    node.expr = AstExpr::Literal(AstLiteral {
                        span: node.span.clone(),
                        value: "0".to_string(),
                        type_: AstLiteralType::I32,
                    });
                }
            }
        }
    }

    fn exit_value(&mut self, node: &mut AstValue) {
        if let AstValueRoot::FnCall(call) = &node.root {
            let statements = inlined_fn_statements(self.analysis, call);
            if !statements.is_empty() {
                self.statements.extend(statements);
                let last_statement = self
                    .statements
                    .pop()
                    .expect("internal error: missing return");
                if let AstStatement::Return(return_) = last_statement {
                    if let AstExpr::Value(value) = return_.expr {
                        node.replace_root(value);
                    }
                } else {
                    self.statements.push(last_statement);
                }
            }
        }
    }
}

fn inlined_fn_statements(analysis: &mut Analysis, call: &AstFnCall) -> Vec<AstStatement> {
    let fn_ = resolver::fn_(analysis, &call.name)
        .expect("internal error: missing function")
        .clone();
    if !fn_.is_inlined {
        return vec![];
    }
    let mut transform = RefFnStatementsTransform::new(analysis, &fn_.ast, call);
    fn_.ast
        .statements
        .into_iter()
        .map(|mut statement| {
            transform.visit_statement(&mut statement);
            statement
        })
        .collect()
}

struct RefFnStatementsTransform<'a> {
    analysis: &'a mut Analysis,
    param_args: FxHashMap<u64, AstExpr>,
    old_new_id: FxHashMap<u64, u64>,
}

impl<'a> RefFnStatementsTransform<'a> {
    fn new(analysis: &'a mut Analysis, fn_: &AstFnItem, call: &AstFnCall) -> Self {
        Self {
            analysis,
            param_args: fn_
                .params
                .iter()
                .zip(&call.args)
                .map(|(param, arg)| (param.name.id, arg.value.clone()))
                .collect(),
            old_new_id: FxHashMap::default(),
        }
    }
}

impl VisitMut for RefFnStatementsTransform<'_> {
    fn enter_value(&mut self, node: &mut AstValue) {
        if let IdentSource::Var(id) = self.analysis.idents[&resolver::value_root_id(node)].source {
            if let Some(AstExpr::Value(new_root)) = self.param_args.get(&id) {
                node.replace_root(new_root.clone());
            }
        }
    }

    fn exit_ident(&mut self, node: &mut AstIdent) {
        if let Some(ident) = self.analysis.idents.get(&node.id) {
            match ident.source {
                IdentSource::Buffer(_) | IdentSource::Fn(_) | IdentSource::Field => {}
                IdentSource::Var(id) => {
                    let ident = ident.clone();
                    let old_id = node.id;
                    node.id = self.analysis.next_id();
                    self.old_new_id.insert(old_id, node.id);
                    self.analysis.idents.insert(
                        node.id,
                        Ident::new(
                            IdentSource::Var(self.old_new_id.get(&id).copied().unwrap_or(id)),
                            ident.type_,
                        ),
                    );
                }
            }
        }
    }
}

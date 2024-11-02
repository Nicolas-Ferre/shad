use crate::{listing, Analysis, Ident, IdentSource};
use fxhash::FxHashMap;
use shad_parser::{
    AstExpr, AstFnCall, AstFnCallStatement, AstFnItem, AstFnQualifier, AstIdent, AstLeftValue,
    AstStatement, VisitMut,
};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    transform_fns(analysis);
    transform_init_blocks(analysis);
    transform_run_blocks(analysis);
}

fn transform_fns(analysis: &mut Analysis) {
    let mut are_fns_inlined: FxHashMap<_, _> = analysis
        .fns
        .iter()
        .map(|(signature, fn_)| (signature.clone(), fn_.ast.qualifier == AstFnQualifier::Gpu))
        .collect();
    let signatures: Vec<_> = analysis.fns.keys().cloned().collect();
    while are_fns_inlined.values().any(|is_inlined| !is_inlined) {
        for signature in &signatures {
            if !are_fns_inlined[signature]
                && are_all_dependent_fns_inlined(analysis, &are_fns_inlined, signature)
            {
                let mut fn_ = analysis.fns[signature].clone();
                visit_statements(analysis, &mut fn_.ast.statements);
                are_fns_inlined.insert(signature.clone(), true);
                analysis.fns.insert(signature.clone(), fn_);
            }
        }
    }
}

fn transform_init_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.init_blocks);
    for block in &mut blocks {
        visit_statements(analysis, &mut block.ast.statements);
    }
    analysis.init_blocks = blocks;
}

fn transform_run_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        visit_statements(analysis, &mut block.ast.statements);
    }
    analysis.run_blocks = blocks;
}

fn are_all_dependent_fns_inlined(
    analysis: &Analysis,
    are_fns_inlined: &FxHashMap<String, bool>,
    signature: &str,
) -> bool {
    analysis.fns[signature]
        .ast
        .statements
        .iter()
        .flat_map(|s| listing::functions::list_in_statement(analysis, s))
        .all(|signature| are_fns_inlined[&signature])
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
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|statement| !is_inline_fn_call_statement(analysis, statement))
        .collect();
}

fn is_inline_fn_call_statement(analysis: &Analysis, statement: &AstStatement) -> bool {
    match statement {
        AstStatement::FnCall(call) => {
            let signature = analysis
                .fn_signature(&call.call.name)
                .expect("internal error: missing signature");
            analysis.fns[&signature].is_inlined
        }
        AstStatement::Assignment(_) | AstStatement::Var(_) | AstStatement::Return(_) => false,
    }
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
    fn exit_left_value(&mut self, node: &mut AstLeftValue) {
        if let AstLeftValue::FnCall(call) = node {
            let statements = inlined_fn_statements(self.analysis, call);
            if !statements.is_empty() {
                self.statements.extend(statements);
                let last_statement = self
                    .statements
                    .pop()
                    .expect("internal error: missing return");
                if let AstStatement::Return(return_) = last_statement {
                    *node = return_
                        .expr
                        .try_into()
                        .expect("internal error: found literal ref");
                } else {
                    unreachable!("internal error: invalid last function statement")
                }
            }
        }
    }

    fn exit_fn_call_statement(&mut self, node: &mut AstFnCallStatement) {
        self.statements
            .extend(inlined_fn_statements(self.analysis, &node.call));
    }

    fn exit_expr(&mut self, node: &mut AstExpr) {
        if let AstExpr::FnCall(call) = node {
            let statements = inlined_fn_statements(self.analysis, call);
            if !statements.is_empty() {
                self.statements.extend(statements);
                let last_statement = self
                    .statements
                    .pop()
                    .expect("internal error: missing return");
                if let AstStatement::Return(return_) = last_statement {
                    *node = return_.expr;
                } else {
                    unreachable!("internal error: invalid last function statement")
                }
            }
        }
    }
}

fn inlined_fn_statements(analysis: &mut Analysis, call: &AstFnCall) -> Vec<AstStatement> {
    let signature = analysis
        .fn_signature(&call.name)
        .expect("internal error: missing signature");
    let fn_ = analysis.fns[&signature].clone();
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
                .map(|(param, arg)| (param.name.id, arg.clone()))
                .collect(),
            old_new_id: FxHashMap::default(),
        }
    }
}

impl VisitMut for RefFnStatementsTransform<'_> {
    fn enter_left_value(&mut self, node: &mut AstLeftValue) {
        if let AstLeftValue::Ident(ident) = node {
            if let IdentSource::Ident(id) = self.analysis.idents[&ident.id].source {
                if let Some(expr) = self.param_args.get(&id) {
                    *node = expr
                        .clone()
                        .try_into()
                        .expect("internal error: invalid literal left value");
                }
            }
        }
    }

    fn enter_expr(&mut self, node: &mut AstExpr) {
        if let AstExpr::Ident(ident) = node {
            if let IdentSource::Ident(id) = self.analysis.idents[&ident.id].source {
                if let Some(expr) = self.param_args.get(&id) {
                    *node = expr.clone();
                }
            }
        }
    }

    fn exit_ident(&mut self, node: &mut AstIdent) {
        if let Some(ident) = self.analysis.idents.get(&node.id) {
            match ident.source {
                IdentSource::Buffer(_) | IdentSource::Fn(_) => {}
                IdentSource::Ident(id) => {
                    let ident = ident.clone();
                    let old_id = node.id;
                    node.id = self.analysis.ast.next_id();
                    self.old_new_id.insert(old_id, node.id);
                    self.analysis.idents.insert(
                        node.id,
                        Ident::new(
                            IdentSource::Ident(self.old_new_id.get(&id).copied().unwrap_or(id)),
                            ident.type_,
                        ),
                    );
                }
            }
        }
    }
}

use crate::{resolver, Analysis, IdentSource};
use fxhash::FxHashMap;
use shad_parser::{AstExpr, AstStatement, AstValue, AstVarDefinition, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    transform_fns(analysis);
    transform_init_blocks(analysis);
    transform_run_blocks(analysis);
}

fn transform_fns(analysis: &mut Analysis) {
    let mut fns = mem::take(&mut analysis.fns);
    for fn_ in fns.values_mut() {
        visit_statements(analysis, &mut fn_.ast.statements);
    }
    analysis.fns = fns;
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

fn visit_statements(analysis: &mut Analysis, statements: &mut Vec<AstStatement>) {
    let mut transform = RefVarInlineTransform::new(analysis);
    *statements = mem::take(statements)
        .into_iter()
        .map(|mut statement| {
            transform.visit_statement(&mut statement);
            statement
        })
        .filter(|statement| {
            if let AstStatement::Var(var_def) = statement {
                !var_def.is_ref
            } else {
                true
            }
        })
        .collect();
}

struct RefVarInlineTransform<'a> {
    analysis: &'a mut Analysis,
    ref_expressions: FxHashMap<u64, AstExpr>,
}

impl<'a> RefVarInlineTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            ref_expressions: FxHashMap::default(),
        }
    }
}

impl VisitMut for RefVarInlineTransform<'_> {
    fn exit_var_definition(&mut self, node: &mut AstVarDefinition) {
        if node.is_ref {
            self.ref_expressions.insert(node.name.id, node.expr.clone());
        }
    }

    fn exit_value(&mut self, node: &mut AstValue) {
        match self.analysis.idents[&resolver::value_root_id(node)].source {
            IdentSource::Var(id) => {
                if let Some(AstExpr::Value(new_root)) = self.ref_expressions.get(&id) {
                    node.replace_root(new_root.clone());
                }
            }
            IdentSource::Buffer(_) | IdentSource::Fn(_) | IdentSource::Field => {}
        }
    }
}

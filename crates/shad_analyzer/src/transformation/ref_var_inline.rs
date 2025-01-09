use crate::{Analysis, Item};
use fxhash::FxHashMap;
use shad_parser::{AstExpr, AstExprRoot, AstStatement, AstVarDefinition, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        let mut transform = RefVarInlineTransform::new(analysis);
        for statement in statements.iter_mut() {
            transform.visit_statement(statement);
        }
        *statements = mem::take(statements)
            .into_iter()
            .filter(is_not_ref_variable_definition)
            .collect();
    });
}

fn is_not_ref_variable_definition(statement: &AstStatement) -> bool {
    if let AstStatement::Var(var_def) = statement {
        !var_def.is_ref
    } else {
        true
    }
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

    fn exit_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let Some(Item::Var(id, _)) = self.analysis.item(ident) {
                if let Some(new_root) = self.ref_expressions.get(&id) {
                    node.replace_root(new_root.clone());
                }
            }
        }
    }
}

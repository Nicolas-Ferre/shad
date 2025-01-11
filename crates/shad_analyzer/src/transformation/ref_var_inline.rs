use crate::Analysis;
use fxhash::FxHashMap;
use shad_parser::{AstExpr, AstExprRoot, AstStatement, AstVarDefinition, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |_analysis, statements| {
        let mut transform = RefVarInlineTransform::default();
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

#[derive(Default)]
struct RefVarInlineTransform {
    ref_expressions: FxHashMap<String, AstExpr>,
}

impl VisitMut for RefVarInlineTransform {
    fn exit_var_definition(&mut self, node: &mut AstVarDefinition) {
        if node.is_ref {
            self.ref_expressions
                .insert(node.name.label.clone(), node.expr.clone());
        }
    }

    fn exit_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let Some(new_root) = self.ref_expressions.get(&ident.label) {
                node.replace_root(new_root.clone());
            }
        }
    }
}

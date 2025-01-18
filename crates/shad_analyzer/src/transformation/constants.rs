use crate::{Analysis, Item};
use shad_parser::{AstExpr, AstExprRoot, AstStatement, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        *statements = mem::take(statements)
            .into_iter()
            .flat_map(|mut statement| {
                let mut transform = ConstantTransform::new(analysis);
                transform.visit_statement(&mut statement);
                transform.statements.push(statement);
                transform.statements
            })
            .collect();
    });
}

struct ConstantTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> ConstantTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }
}

impl VisitMut for ConstantTransform<'_> {
    fn enter_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let Some(Item::Constant(constant)) = self.analysis.item(ident) {
                let value = constant
                    .value
                    .clone()
                    .expect("internal error: not calculated constant");
                node.root = AstExprRoot::Literal(value.literal(&ident.span));
            }
        }
    }
}

use crate::Analysis;
use shad_parser::{AstLiteral, VisitMut};

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |_, statements| {
        for statement in statements {
            LiteralTransform.visit_statement(statement);
        }
    });
    for constant in analysis.constants.values_mut() {
        LiteralTransform.visit_const_item(&mut constant.ast);
    }
}

struct LiteralTransform;

impl VisitMut for LiteralTransform {
    fn visit_literal(&mut self, node: &mut AstLiteral) {
        node.value = node.value.replace('_', "");
    }
}

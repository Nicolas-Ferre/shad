use crate::Analysis;
use shad_parser::{AstLiteral, VisitMut};

pub(crate) fn transform(analysis: &mut Analysis) {
    for block in &mut analysis.init_blocks {
        LiteralTransform.visit_run_item(&mut block.ast);
    }
    for block in &mut analysis.run_blocks {
        LiteralTransform.visit_run_item(&mut block.ast);
    }
    for fn_ in analysis.fns.values_mut() {
        LiteralTransform.visit_fn_item(&mut fn_.ast);
    }
}

struct LiteralTransform;

impl VisitMut for LiteralTransform {
    fn visit_literal(&mut self, node: &mut AstLiteral) {
        node.value = node.value.replace('_', "");
    }
}

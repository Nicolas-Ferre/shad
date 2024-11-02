use crate::Analysis;
use shad_parser::{AstAssignment, AstItem, AstLeftValue, AstRunItem, AstStatement};

#[derive(Debug, Clone)]
pub struct RunBlock {
    pub ast: AstRunItem,
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    for item in &analysis.ast.items {
        if let AstItem::Buffer(buffer) = item {
            analysis.init_blocks.push(RunBlock {
                ast: AstRunItem {
                    statements: vec![AstStatement::Assignment(AstAssignment {
                        span: buffer.value.span(),
                        value: AstLeftValue::Ident(buffer.name.clone()),
                        expr: buffer.value.clone(),
                    })],
                },
            });
        }
    }
}

fn register_steps(analysis: &mut Analysis) {
    for item in &analysis.ast.items {
        if let AstItem::Run(block) = item {
            analysis.run_blocks.push(RunBlock { ast: block.clone() });
        }
    }
}
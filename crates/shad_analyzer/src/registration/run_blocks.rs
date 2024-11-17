use crate::Analysis;
use shad_parser::{AstAssignment, AstItem, AstLeftValue, AstRunItem, AstStatement};

/// An analyzed block of statements.
#[derive(Debug, Clone)]
pub struct RunBlock {
    /// The block AST.
    pub ast: AstRunItem,
    /// The module where the block comes from.
    pub module: String,
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    for (id, buffer_id) in analysis.ordered_buffers.iter().enumerate() {
        let buffer = &analysis.buffers[buffer_id];
        analysis.init_blocks.push(RunBlock {
            ast: AstRunItem {
                statements: vec![AstStatement::Assignment(AstAssignment {
                    span: buffer.ast.value.span().clone(),
                    value: AstLeftValue::Ident(buffer.ast.name.clone()),
                    expr: buffer.ast.value.clone(),
                })],
                id: id as u64,
            },
            module: buffer_id.module.clone(),
        });
    }
}

fn register_steps(analysis: &mut Analysis) {
    for (module, ast) in &analysis.asts {
        for item in &ast.items {
            if let AstItem::Run(block) = item {
                analysis.run_blocks.push(RunBlock {
                    ast: block.clone(),
                    module: module.clone(),
                });
            }
        }
    }
}

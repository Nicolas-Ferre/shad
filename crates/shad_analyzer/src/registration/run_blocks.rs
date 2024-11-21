use crate::Analysis;
use shad_parser::{AstAssignment, AstItem, AstLeftValue, AstRunItem, AstStatement};

/// An analyzed block of statements.
#[derive(Debug, Clone)]
pub struct BufferInitRunBlock {
    /// The block AST.
    pub ast: AstRunItem,
    // TODO: use BufferId ?
    /// The module where the block comes from.
    pub module: String,
    /// The buffer name.
    pub buffer: String,
}

/// An analyzed block of statements.
#[derive(Debug, Clone)]
pub struct RunBlock {
    /// The block AST.
    pub ast: AstRunItem,
    /// The module where the block comes from.
    pub module: String,
}

impl RunBlock {
    pub(crate) fn priority(&self) -> i32 {
        self.ast.priority.unwrap_or(0)
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    for (module, ast) in &analysis.asts {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                analysis.init_blocks.push(BufferInitRunBlock {
                    ast: AstRunItem {
                        statements: vec![AstStatement::Assignment(AstAssignment {
                            span: buffer.value.span().clone(),
                            value: AstLeftValue::Ident(buffer.name.clone()),
                            expr: buffer.value.clone(),
                        })],
                        priority: None,
                        id: 0,
                    },
                    module: module.clone(),
                    buffer: buffer.name.label.clone(),
                });
            }
        }
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

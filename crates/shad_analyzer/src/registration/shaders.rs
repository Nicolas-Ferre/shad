use crate::listing::{buffers, functions};
use crate::{Analysis, RunBlock};
use shad_parser::AstStatement;

#[derive(Debug, Clone)]
pub struct ComputeShader {
    pub buffers: Vec<String>,
    pub fn_signatures: Vec<String>,
    pub statements: Vec<AstStatement>,
}

impl ComputeShader {
    fn new(analysis: &Analysis, block: &RunBlock) -> Self {
        Self {
            buffers: buffers::list(analysis, &block.ast),
            fn_signatures: functions::list_in_block(analysis, &block.ast),
            statements: block.ast.statements.clone(),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    for block in &analysis.init_blocks {
        let shader = ComputeShader::new(analysis, block);
        analysis.init_shaders.push(shader);
    }
}

fn register_steps(analysis: &mut Analysis) {
    for block in &analysis.run_blocks {
        let shader = ComputeShader::new(analysis, block);
        analysis.step_shaders.push(shader);
    }
}

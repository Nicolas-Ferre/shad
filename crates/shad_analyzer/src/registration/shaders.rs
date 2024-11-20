use crate::listing::{buffers, functions};
use crate::{Analysis, BufferId, FnId, RunBlock};
use itertools::Itertools;
use shad_parser::AstStatement;

/// An analyzed compute shader.
#[derive(Debug, Clone)]
pub struct ComputeShader {
    /// The buffers used by the shader.
    pub buffers: Vec<BufferId>,
    /// The identifiers of the functions used by the shader.
    pub fn_ids: Vec<FnId>,
    /// The statements of the shader.
    pub statements: Vec<AstStatement>,
}

impl ComputeShader {
    fn new(analysis: &Analysis, block: &RunBlock) -> Self {
        Self {
            buffers: buffers::list_in_block(analysis, &block.ast),
            fn_ids: functions::list_in_block(analysis, &block.ast),
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
    for block in analysis
        .run_blocks
        .iter()
        .sorted_unstable_by_key(|block| (-block.priority(), &block.module, block.ast.id))
    {
        let shader = ComputeShader::new(analysis, block);
        analysis.step_shaders.push(shader);
    }
}

use crate::{listing, Analysis, BufferId, SpecializedFn, TypeId};
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use shad_parser::{AstRunItem, AstStatement};

/// An analyzed compute shader.
#[derive(Debug, Clone)]
pub struct ComputeShader {
    /// The buffers IDs used by the shader.
    pub buffer_ids: Vec<BufferId>,
    /// The function used by the shader.
    pub fns: Vec<SpecializedFn>,
    /// The type IDs used by the shader.
    pub type_ids: Vec<TypeId>,
    /// The statements of the shader.
    pub statements: Vec<AstStatement>,
}

impl ComputeShader {
    fn new(analysis: &Analysis, block: &AstRunItem) -> Self {
        Self {
            buffer_ids: listing::buffers::list_in_block(analysis, block),
            fns: listing::functions::list_in_block(analysis, block),
            type_ids: listing::types::list_in_block(analysis, block),
            statements: block.statements.clone(),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    let dependent_buffers = find_dependent_buffers(analysis);
    let buffer_blocks: FxHashMap<_, _> = analysis
        .init_blocks
        .iter()
        .map(|block| (&block.buffer, block))
        .collect();
    let mut initialized_buffers = FxHashSet::default();
    while initialized_buffers.len() < dependent_buffers.len() {
        for (buffer, dependencies) in &dependent_buffers {
            if dependencies
                .iter()
                .all(|buffer| initialized_buffers.contains(buffer))
            {
                let block = &buffer_blocks[buffer];
                let shader = ComputeShader::new(analysis, &block.ast);
                analysis.init_shaders.push(shader);
                initialized_buffers.insert(buffer.clone());
            }
        }
    }
}

fn register_steps(analysis: &mut Analysis) {
    for block in analysis
        .run_blocks
        .iter()
        .sorted_unstable_by_key(|block| (-block.priority(), &block.module, block.ast.id))
    {
        let shader = ComputeShader::new(analysis, &block.ast);
        analysis.step_shaders.push(shader);
    }
}

fn find_dependent_buffers(analysis: &Analysis) -> FxHashMap<BufferId, FxHashSet<BufferId>> {
    analysis
        .init_blocks
        .iter()
        .map(|block| {
            (
                block.buffer.clone(),
                listing::buffers::list_in_block(analysis, &block.ast)
                    .into_iter()
                    .filter(|buffer| &block.buffer != buffer)
                    .collect::<FxHashSet<_>>(),
            )
        })
        .collect()
}

use crate::{listing, Analysis, BufferId, FnId};
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use shad_parser::{AstRunItem, AstStatement};
use std::cmp::Ordering;

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
    fn new(analysis: &Analysis, block: &AstRunItem) -> Self {
        Self {
            buffers: listing::buffers::list_in_block(analysis, block),
            fn_ids: listing::functions::list_in_block(analysis, block),
            statements: block.statements.clone(),
        }
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_init(analysis);
    register_steps(analysis);
}

fn register_init(analysis: &mut Analysis) {
    let dependent_buffers: FxHashMap<_, _> = find_dependent_buffers(analysis);
    for (block, _) in analysis
        .init_blocks
        .iter()
        .map(|block| (block, &block.buffer))
        .sorted_unstable_by(|(_, id1), (_, id2)| {
            if dependent_buffers[id1].contains(id2) {
                Ordering::Greater
            } else if dependent_buffers[id2].contains(id1) {
                Ordering::Less
            } else {
                id1.cmp(id2)
            }
        })
    {
        let shader = ComputeShader::new(analysis, &block.ast);
        analysis.init_shaders.push(shader);
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

fn find_dependent_buffers(analysis: &Analysis) -> FxHashMap<&BufferId, FxHashSet<BufferId>> {
    analysis
        .init_blocks
        .iter()
        .map(|block| {
            (
                &block.buffer,
                listing::buffers::list_in_block(analysis, &block.ast)
                    .into_iter()
                    .collect::<FxHashSet<_>>(),
            )
        })
        .collect()
}

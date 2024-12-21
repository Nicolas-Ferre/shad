use crate::Analysis;
use shad_parser::AstStatement;
use std::mem;

pub(crate) mod expr_statements;
pub(crate) mod fn_params;
pub(crate) mod literals;
pub(crate) mod ref_fn_inline;
pub(crate) mod ref_split;
pub(crate) mod ref_var_inline;
pub(crate) mod values;

const GENERATED_IDENT_LABEL: &str = "generated";

fn transform_statements(
    analysis: &mut Analysis,
    transform_fn: impl Fn(&mut Analysis, &mut Vec<AstStatement>),
) {
    transform_init_blocks(analysis, &transform_fn);
    transform_run_blocks(analysis, &transform_fn);
    transform_fns(analysis, &transform_fn);
}

fn transform_init_blocks(
    analysis: &mut Analysis,
    transform_fn: impl Fn(&mut Analysis, &mut Vec<AstStatement>),
) {
    let mut blocks = mem::take(&mut analysis.init_blocks);
    for block in &mut blocks {
        transform_fn(analysis, &mut block.ast.statements);
    }
    analysis.init_blocks = blocks;
}

fn transform_run_blocks(
    analysis: &mut Analysis,
    transform_fn: impl Fn(&mut Analysis, &mut Vec<AstStatement>),
) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        transform_fn(analysis, &mut block.ast.statements);
    }
    analysis.run_blocks = blocks;
}

fn transform_fns(
    analysis: &mut Analysis,
    transform_fn: impl Fn(&mut Analysis, &mut Vec<AstStatement>),
) {
    let mut fns = analysis.fns.clone();
    for fn_ in fns.values_mut() {
        transform_fn(analysis, &mut fn_.ast.statements);
    }
    analysis.fns = fns;
}

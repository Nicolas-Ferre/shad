use crate::{resolving, Analysis, Ident, IdentSource};
use shad_parser::{AstExpr, AstIdent, AstStatement, AstVarDefinition};
use std::mem;

pub(crate) mod constants;
pub(crate) mod expr_statements;
pub(crate) mod fn_params;
pub(crate) mod left_values;
pub(crate) mod literals;
pub(crate) mod ref_fn_inline;
pub(crate) mod ref_split;
pub(crate) mod ref_var_inline;

fn extract_in_variable(
    analysis: &mut Analysis,
    expr: &AstExpr,
    is_ref: bool,
) -> (AstStatement, AstIdent) {
    let type_id = resolving::types::expr(analysis, expr);
    let var_name = "generated";
    let var_def_id = analysis.next_id();
    let var_id = analysis.next_id();
    analysis.idents.insert(
        var_def_id,
        Ident {
            source: IdentSource::Var(var_def_id),
            type_id: type_id.clone(),
        },
    );
    analysis.idents.insert(
        var_id,
        Ident {
            source: IdentSource::Var(var_def_id),
            type_id,
        },
    );
    (
        AstStatement::Var(AstVarDefinition {
            span: expr.span.clone(),
            name: AstIdent {
                span: expr.span.clone(),
                label: var_name.to_string(),
                id: var_def_id,
            },
            is_ref,
            expr: expr.clone(),
        }),
        AstIdent {
            span: expr.span.clone(),
            label: var_name.to_string(),
            id: var_id,
        },
    )
}

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

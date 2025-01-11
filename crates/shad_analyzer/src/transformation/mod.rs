use crate::registration::vars::Var;
use crate::{resolving, Analysis};
use shad_parser::{AstExpr, AstIdent, AstIdentKind, AstStatement, AstVarDefinition};
use std::mem;

pub(crate) mod constants;
pub(crate) mod expr_statements;
pub(crate) mod fn_params;
pub(crate) mod left_values;
pub(crate) mod ref_fn_inline;
pub(crate) mod ref_split;
pub(crate) mod ref_var_inline;

// An identifier character valid in WGSL but not in Shad,
// to ensure generated identifiers don't conflict with Shad identifiers defined by users.
const SPECIAL_WGSL_IDENT_CHARACTER: char = 'µ';

fn extract_in_variable(
    analysis: &mut Analysis,
    expr: &AstExpr,
    is_ref: bool,
) -> (AstStatement, AstIdent) {
    let type_id = resolving::types::expr(analysis, expr);
    let var_id = analysis.next_id();
    let var_name = format!("generated{SPECIAL_WGSL_IDENT_CHARACTER}{var_id}");
    analysis.vars.insert(var_id, Var { type_id });
    (
        AstStatement::Var(AstVarDefinition {
            span: expr.span.clone(),
            name: AstIdent {
                span: expr.span.clone(),
                label: var_name.clone(),
                id: var_id,
                kind: AstIdentKind::VarDef,
            },
            is_ref,
            expr: expr.clone(),
        }),
        AstIdent {
            span: expr.span.clone(),
            label: var_name,
            id: var_id,
            kind: AstIdentKind::Other,
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

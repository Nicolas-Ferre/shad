use crate::transformation::GENERATED_IDENT_LABEL;
use crate::{resolver, Analysis, Ident, IdentSource};
use shad_parser::{AstExprStatement, AstIdent, AstStatement, AstVarDefinition};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    transform_init_blocks(analysis);
    transform_run_blocks(analysis);
    transform_fns(analysis);
}

fn transform_init_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.init_blocks);
    for block in &mut blocks {
        visit_statements(analysis, &mut block.ast.statements);
    }
    analysis.init_blocks = blocks;
}

fn transform_run_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        visit_statements(analysis, &mut block.ast.statements);
    }
    analysis.run_blocks = blocks;
}

fn transform_fns(analysis: &mut Analysis) {
    let mut fns = analysis.fns.clone();
    for fn_ in fns.values_mut() {
        visit_statements(analysis, &mut fn_.ast.statements);
    }
    analysis.fns = fns;
}

fn visit_statements(analysis: &mut Analysis, statements: &mut Vec<AstStatement>) {
    *statements = mem::take(statements)
        .into_iter()
        .map(|statement| {
            if let AstStatement::Expr(call) = statement {
                transform_expr(analysis, call)
            } else {
                statement
            }
        })
        .collect();
}

fn transform_expr(analysis: &mut Analysis, statement: AstExprStatement) -> AstStatement {
    if let Some(expr_type) = resolver::expr_type(analysis, &statement.expr) {
        let id = analysis.next_id();
        analysis.idents.insert(
            id,
            Ident {
                source: IdentSource::Var(id),
                type_: Some(expr_type),
            },
        );
        AstStatement::Var(AstVarDefinition {
            span: statement.span.clone(),
            name: AstIdent {
                span: statement.span.clone(),
                label: GENERATED_IDENT_LABEL.into(),
                id,
            },
            is_ref: false,
            expr: statement.expr.clone(),
        })
    } else {
        AstStatement::Expr(statement)
    }
}

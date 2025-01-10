use crate::{resolving, Analysis, NO_RETURN_TYPE};
use shad_parser::{AstExprStatement, AstStatement};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
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
    });
}

fn transform_expr(analysis: &mut Analysis, statement: AstExprStatement) -> AstStatement {
    let has_type = resolving::types::expr(analysis, &statement.expr)
        .map_or(false, |type_id| type_id.name != NO_RETURN_TYPE);
    if has_type {
        let (var_def_statement, _var_name) =
            super::extract_in_variable(analysis, &statement.expr, false);
        var_def_statement
    } else {
        AstStatement::Expr(statement)
    }
}

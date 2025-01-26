use crate::{atoms, IDENT_UNIT};
use itertools::Itertools;
use shad_analyzer::{Analysis, GenericValue};
use shad_parser::AstStatement;
use std::mem;

pub(crate) fn to_wgsl(
    analysis: &Analysis,
    statements: &[AstStatement],
    mut generic_values: Vec<(String, GenericValue)>,
) -> String {
    statements
        .iter()
        .map(|statement| to_statement_wgsl(analysis, statement, &mut generic_values, 1))
        .join("\n")
}

fn to_statement_wgsl(
    analysis: &Analysis,
    statement: &AstStatement,
    generic_values: &mut Vec<(String, GenericValue)>,
    indent: usize,
) -> String {
    match statement {
        AstStatement::Var(statement) => {
            *generic_values = mem::take(generic_values)
                .into_iter()
                .filter(|(name, _)| name != &statement.name.label)
                .collect();
            format!(
                "{empty: >width$}var {} = {};",
                atoms::to_var_ident_wgsl(analysis, &statement.name, generic_values),
                atoms::to_expr_wgsl(analysis, &statement.expr, generic_values),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Assignment(statement) => {
            format!(
                "{empty: >width$}{} = {};",
                atoms::to_expr_wgsl(analysis, &statement.left, generic_values),
                atoms::to_expr_wgsl(analysis, &statement.right, generic_values),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Return(statement) => {
            format!(
                "{empty: >width$}return {};",
                atoms::to_expr_wgsl(analysis, &statement.expr, generic_values),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Expr(statement) => {
            format!(
                "{empty: >width$}{};",
                atoms::to_expr_wgsl(analysis, &statement.expr, generic_values),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

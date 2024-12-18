use crate::{atoms, IDENT_UNIT};
use itertools::Itertools;
use shad_analyzer::Analysis;
use shad_parser::AstStatement;

pub(crate) fn to_wgsl(analysis: &Analysis, statements: &[AstStatement]) -> String {
    statements
        .iter()
        .map(|statement| to_statement_wgsl(analysis, statement, 1))
        .join("\n")
}

fn to_statement_wgsl(analysis: &Analysis, statement: &AstStatement, indent: usize) -> String {
    match statement {
        AstStatement::Var(statement) => {
            format!(
                "{empty: >width$}var {} = {};",
                atoms::to_ident_wgsl(analysis, &statement.name),
                atoms::to_expr_wgsl(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Assignment(statement) => {
            format!(
                "{empty: >width$}{} = {};",
                atoms::to_value_wgsl(analysis, &statement.value),
                atoms::to_expr_wgsl(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Return(statement) => {
            format!(
                "{empty: >width$}return {};",
                atoms::to_expr_wgsl(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Expr(statement) => {
            format!(
                "{empty: >width$}{};",
                atoms::to_expr_wgsl(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

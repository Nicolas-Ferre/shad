use crate::{atoms, fn_calls, IDENT_UNIT};
use itertools::Itertools;
use shad_analyzer::Analysis;
use shad_parser::{AstLeftValue, AstStatement};

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
        AstStatement::Assignment(statement) => match &statement.value {
            AstLeftValue::IdentPath(assigned) => {
                format!(
                    "{empty: >width$}{} = {};",
                    atoms::to_ident_path_wgsl(analysis, assigned),
                    atoms::to_expr_wgsl(analysis, &statement.expr),
                    empty = "",
                    width = indent * IDENT_UNIT,
                )
            }
            AstLeftValue::FnCall(_) => unreachable!("internal error: invalid inlining"),
        },
        AstStatement::Return(statement) => {
            format!(
                "{empty: >width$}return {};",
                atoms::to_expr_wgsl(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::FnCall(statement) => {
            format!(
                "{empty: >width$}{};",
                fn_calls::to_wgsl(analysis, &statement.call),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

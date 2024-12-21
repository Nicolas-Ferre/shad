use crate::transformation::GENERATED_IDENT_LABEL;
use crate::{resolver, Analysis, Ident, IdentSource};
use shad_parser::{AstExprStatement, AstIdent, AstStatement, AstVarDefinition};
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

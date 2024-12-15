use crate::transformation::GENERATED_IDENT_LABEL;
use crate::{resolver, Analysis, Ident, IdentSource};
use shad_parser::{
    AstExpr, AstFnCallStatement, AstFnQualifier, AstIdent, AstStatement, AstVarDefinition,
};
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
            if let AstStatement::FnCall(call) = statement {
                transform_call_statement(analysis, call)
            } else {
                statement
            }
        })
        .collect();
}

fn transform_call_statement(analysis: &mut Analysis, call: AstFnCallStatement) -> AstStatement {
    if let Some(fn_) = resolver::fn_(analysis, &call.call.name) {
        if fn_.ast.qualifier == AstFnQualifier::Gpu && fn_.return_type_id.is_some() {
            let type_ = fn_.return_type_id.clone();
            let id = analysis.next_id();
            analysis.idents.insert(
                id,
                Ident {
                    source: IdentSource::Var(id),
                    type_,
                },
            );
            return AstStatement::Var(AstVarDefinition {
                span: call.span.clone(),
                name: AstIdent {
                    span: call.span.clone(),
                    label: GENERATED_IDENT_LABEL.into(),
                    id,
                },
                is_ref: false,
                expr: AstExpr::Value(call.call.into()),
            });
        }
    }
    AstStatement::FnCall(call)
}

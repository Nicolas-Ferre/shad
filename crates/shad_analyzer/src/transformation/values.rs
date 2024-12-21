use crate::{resolver, Analysis};
use shad_parser::{AstAssignment, AstExpr, AstStatement, AstValueRoot, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        *statements = mem::take(statements)
            .into_iter()
            .flat_map(|mut statement| {
                let mut transform = ValueTransform::new(analysis);
                transform.visit_statement(&mut statement);
                transform.statements.push(statement);
                transform.statements
            })
            .collect();
    });
}

struct ValueTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> ValueTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }
}

impl VisitMut for ValueTransform<'_> {
    fn enter_assignment(&mut self, node: &mut AstAssignment) {
        if let AstValueRoot::FnCall(call) = &node.value.root {
            if let Some(fn_) = resolver::fn_(self.analysis, &call.name) {
                let is_ref = fn_
                    .ast
                    .return_type
                    .as_ref()
                    .map_or(false, |type_| type_.is_ref);
                let (var_def_statement, var_name) = super::extract_in_variable(
                    self.analysis,
                    &AstExpr::Value(call.clone().into()),
                    is_ref,
                );
                self.statements.push(var_def_statement);
                node.value.root = AstValueRoot::Ident(var_name);
            }
        }
    }
}

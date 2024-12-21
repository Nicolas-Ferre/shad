use crate::{resolver, Analysis, Ident, IdentSource};
use shad_parser::{
    AstAssignment, AstExpr, AstIdent, AstStatement, AstValueRoot, AstVarDefinition, VisitMut,
};
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
                let return_type_id = fn_.return_type_id.clone();
                let var_name = "call".to_string();
                let var_def_id = self.analysis.next_id();
                let var_id = self.analysis.next_id();
                self.statements.push(AstStatement::Var(AstVarDefinition {
                    span: node.span.clone(),
                    name: AstIdent {
                        span: node.span.clone(),
                        label: var_name.clone(),
                        id: var_def_id,
                    },
                    is_ref,
                    expr: AstExpr::Value(call.clone().into()),
                }));
                node.value.root = AstValueRoot::Ident(AstIdent {
                    span: node.span.clone(),
                    label: var_name,
                    id: var_id,
                });
                self.analysis.idents.insert(
                    var_def_id,
                    Ident {
                        source: IdentSource::Var(var_def_id),
                        type_: return_type_id.clone(),
                    },
                );
                self.analysis.idents.insert(
                    var_id,
                    Ident {
                        source: IdentSource::Var(var_def_id),
                        type_: return_type_id,
                    },
                );
            }
        }
    }
}

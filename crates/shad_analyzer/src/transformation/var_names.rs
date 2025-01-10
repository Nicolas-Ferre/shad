use crate::Analysis;
use fxhash::FxHashMap;
use shad_parser::{AstIdent, AstIdentKind, AstStatement, AstVarDefinition, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, transform_statements);
}

pub(crate) fn transform_statements(analysis: &mut Analysis, statements: &mut Vec<AstStatement>) {
    let mut transform = VarNameTransform::new(analysis);
    let mut transformed_statements = mem::take(statements);
    for statement in &mut transformed_statements {
        transform.visit_statement(statement);
    }
    *statements = transformed_statements;
}

struct VarNameTransform<'a> {
    analysis: &'a mut Analysis,
    var_ids: FxHashMap<String, u64>,
}

impl<'a> VarNameTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            var_ids: FxHashMap::default(),
        }
    }
}

impl VisitMut for VarNameTransform<'_> {
    fn exit_var_definition(&mut self, node: &mut AstVarDefinition) {
        let id = self.analysis.next_id();
        self.var_ids.insert(node.name.label.clone(), id);
        node.name.label = format!("{}_{id}", node.name.label);
    }

    fn exit_ident(&mut self, node: &mut AstIdent) {
        match node.kind {
            AstIdentKind::Other => {
                if let Some(id) = self.var_ids.get(&node.label) {
                    node.label = format!("{}_{id}", node.label);
                }
            }
            AstIdentKind::VarDef | AstIdentKind::FnRef | AstIdentKind::FieldRef => (),
        }
    }
}

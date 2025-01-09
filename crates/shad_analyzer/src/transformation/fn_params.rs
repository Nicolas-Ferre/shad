use crate::Analysis;
use shad_parser::{AstFnItem, AstIdent, AstIdentKind, AstStatement, AstVarDefinition, VisitMut};
use std::mem;

// Defines a variable for each function parameter, so that parameters can be mutated in WGSL.
pub(crate) fn transform(analysis: &mut Analysis) {
    let mut fns = mem::take(&mut analysis.fns);
    for fn_ in fns.values_mut() {
        if !fn_.is_inlined && fn_.ast.gpu_qualifier.is_none() {
            FnParamTransform::new(analysis).visit_fn_item(&mut fn_.ast);
        }
    }
    analysis.fns = fns;
}

struct FnParamTransform<'a> {
    analysis: &'a mut Analysis,
}

impl<'a> FnParamTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self { analysis }
    }
}

impl VisitMut for FnParamTransform<'_> {
    fn enter_fn_item(&mut self, node: &mut AstFnItem) {
        for (index, param) in node.params.iter().enumerate() {
            node.statements.insert(
                index,
                AstStatement::Var(AstVarDefinition {
                    span: param.name.span.clone(),
                    name: AstIdent {
                        span: param.name.span.clone(),
                        label: param.name.label.clone(),
                        id: self.analysis.next_id(),
                        kind: AstIdentKind::Other,
                    },
                    is_ref: false,
                    expr: AstIdent {
                        span: param.name.span.clone(),
                        label: param.name.label.clone(),
                        id: param.name.id,
                        kind: AstIdentKind::Other,
                    }
                    .into(),
                }),
            );
        }
    }
}

use crate::resolving::expressions::ExprSemantic;
use crate::{resolving, Analysis};
use shad_parser::{AstFnCall, AstStatement, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        *statements = mem::take(statements)
            .into_iter()
            .flat_map(|mut statement| {
                let mut transform = RefSplitTransform::new(analysis);
                transform.visit_statement(&mut statement);
                transform.statements.push(statement);
                transform.statements
            })
            .collect();
    });
}

struct RefSplitTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> RefSplitTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }
}

impl VisitMut for RefSplitTransform<'_> {
    fn exit_fn_call(&mut self, node: &mut AstFnCall) {
        if let Some(fn_) = resolving::items::fn_(self.analysis, node, false) {
            if !fn_.is_inlined {
                return;
            }
            let fn_ = fn_.clone();
            for (param, arg) in fn_.ast.params.iter().zip(&mut node.args) {
                if param.ref_span.is_none()
                    || resolving::expressions::semantic(self.analysis, &arg.value)
                        != ExprSemantic::Ref
                {
                    let (var_def_statement, var_name) =
                        super::extract_in_variable(self.analysis, &arg.value, false);
                    self.statements.push(var_def_statement);
                    arg.value = var_name.into();
                }
            }
        } else {
            unreachable!("internal error: missing function");
        }
    }
}

use crate::registration::types;
use crate::{Analysis, Ident, IdentSource};
use shad_parser::{
    AstExpr, AstFnCall, AstIdent, AstIdentType, AstStatement, AstVarDefinition, VisitMut,
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
        visit_statements(analysis, &block.buffer.module, &mut block.ast.statements);
    }
    analysis.init_blocks = blocks;
}

fn transform_run_blocks(analysis: &mut Analysis) {
    let mut blocks = mem::take(&mut analysis.run_blocks);
    for block in &mut blocks {
        visit_statements(analysis, &block.module, &mut block.ast.statements);
    }
    analysis.run_blocks = blocks;
}

fn transform_fns(analysis: &mut Analysis) {
    let mut fns = analysis.fns.clone();
    for fn_ in fns.values_mut() {
        let module = &fn_.ast.name.span.module.name;
        visit_statements(analysis, module, &mut fn_.ast.statements);
    }
    analysis.fns = fns;
}

fn visit_statements(analysis: &mut Analysis, module: &str, statements: &mut Vec<AstStatement>) {
    *statements = mem::take(statements)
        .into_iter()
        .flat_map(|mut statement| {
            let mut transform = RefSplitTransform::new(analysis, module);
            transform.visit_statement(&mut statement);
            transform.statements.push(statement);
            transform.statements
        })
        .collect();
}

struct RefSplitTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
    module: &'a str,
}

impl<'a> RefSplitTransform<'a> {
    fn new(analysis: &'a mut Analysis, module: &'a str) -> Self {
        Self {
            analysis,
            statements: vec![],
            module,
        }
    }
}

impl VisitMut for RefSplitTransform<'_> {
    fn exit_fn_call(&mut self, node: &mut AstFnCall) {
        let fn_id = self
            .analysis
            .fn_id(&node.name)
            .expect("internal error: missing function");
        let fn_ = self.analysis.fns[&fn_id].clone();
        if !fn_.is_inlined {
            return;
        }
        for (param, arg) in fn_.ast.params.iter().zip(&mut node.args) {
            if param.ref_span.is_some() {
                continue;
            }
            let var_label = "tmp";
            let var_def_id = self.analysis.next_id();
            let var_usage_id = self.analysis.next_id();
            let arg = mem::replace(
                arg,
                AstExpr::Ident(AstIdent {
                    span: arg.span().clone(),
                    label: var_label.into(),
                    id: var_usage_id,
                    type_: AstIdentType::VarUsage,
                }),
            );
            self.statements.push(AstStatement::Var(AstVarDefinition {
                span: arg.span().clone(),
                name: AstIdent {
                    span: arg.span().clone(),
                    label: var_label.into(),
                    id: var_def_id,
                    type_: AstIdentType::VarDef,
                },
                expr: arg,
            }));
            let type_id = types::find(self.analysis, self.module, &param.type_)
                .expect("internal error: invalid type");
            self.analysis.idents.insert(
                var_def_id,
                Ident::new(IdentSource::Var(var_def_id), Some(type_id.clone())),
            );
            self.analysis.idents.insert(
                var_usage_id,
                Ident::new(IdentSource::Var(var_def_id), Some(type_id)),
            );
        }
    }
}

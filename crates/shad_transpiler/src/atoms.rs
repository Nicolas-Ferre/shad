use crate::fn_calls;
use itertools::Itertools;
use shad_analyzer::{Analysis, BufferId, IdentSource, TypeId};
use shad_parser::{AstExpr, AstFnQualifier, AstIdent, AstIdentPath};

pub(crate) fn to_expr_wgsl(analysis: &Analysis, expr: &AstExpr) -> String {
    match expr {
        AstExpr::Literal(expr) => match expr.value.as_str() {
            "false" => "0u".into(),
            "true" => "1u".into(),
            _ => expr.value.clone(),
        },
        AstExpr::IdentPath(expr) => to_ident_path_wgsl(analysis, expr),
        AstExpr::FnCall(expr) => fn_calls::to_wgsl(analysis, expr),
    }
}

pub(crate) fn to_ident_path_wgsl(analysis: &Analysis, path: &AstIdentPath) -> String {
    path.segments
        .iter()
        .map(|ident| to_ident_wgsl(analysis, ident))
        .join(".")
}

pub(crate) fn to_ident_wgsl(analysis: &Analysis, name: &AstIdent) -> String {
    match &analysis.idents[&name.id].source {
        IdentSource::Buffer(name) => to_buffer_ident_wgsl(analysis, name),
        IdentSource::Var(id) => format!("v{}_{}", id, name.label),
        IdentSource::Fn(_) => {
            let fn_ = analysis.fn_(name).expect("internal error: missing fn");
            if fn_.ast.qualifier == AstFnQualifier::Gpu {
                if let Some(source_type) = &fn_.source_type {
                    to_type_wgsl(analysis, source_type)
                } else {
                    fn_.ast.name.label.clone()
                }
            } else {
                format!("f{}_{}", fn_.ast.name.id, fn_.ast.name.label)
            }
        }
        IdentSource::Field => format!("s{}", name.label),
    }
}

pub(crate) fn to_buffer_ident_wgsl(analysis: &Analysis, buffer: &BufferId) -> String {
    let name = &analysis.buffers[buffer].ast.name;
    format!("b{}_{}", name.id, name.label)
}

pub(crate) fn to_type_wgsl(analysis: &Analysis, type_id: &TypeId) -> String {
    let type_ = &analysis.types[type_id];
    if let Some(type_) = &type_.ast {
        format!("t{}_{}", type_.name.id, type_.name.label)
    } else if type_.id == TypeId::from_builtin("bool") {
        "u32".into()
    } else {
        type_.name.clone()
    }
}

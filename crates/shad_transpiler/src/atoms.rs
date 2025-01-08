use crate::fn_calls;
use itertools::Itertools;
use shad_analyzer::{Analysis, BufferId, IdentSource, TypeId};
use shad_parser::{AstExpr, AstExprRoot, AstGpuGenericParam, AstGpuName, AstIdent, AstLiteralType};
use std::iter;

pub(crate) fn to_expr_wgsl(analysis: &Analysis, expr: &AstExpr) -> String {
    let root = match &expr.root {
        AstExprRoot::Ident(ident) => to_ident_wgsl(analysis, ident),
        AstExprRoot::FnCall(call) => fn_calls::to_wgsl(analysis, call),
        AstExprRoot::Literal(expr) => {
            let value = match expr.cleaned_value.as_str() {
                "false" => "0u".into(),
                "true" => "1u".into(),
                _ => expr.cleaned_value.clone(),
            };
            let converter = match expr.type_ {
                AstLiteralType::F32 => "f32",
                AstLiteralType::I32 => "i32",
                AstLiteralType::U32 | AstLiteralType::Bool => "u32",
            };
            format!("{converter}({value})")
        }
    };
    let fields = expr
        .fields
        .iter()
        .map(|ident| to_ident_wgsl(analysis, ident));
    iter::once(root).chain(fields).join(".")
}

pub(crate) fn to_ident_wgsl(analysis: &Analysis, name: &AstIdent) -> String {
    let ident = &analysis.idents[&name.id];
    match &ident.source {
        IdentSource::Buffer(name) => to_buffer_ident_wgsl(analysis, name),
        IdentSource::Var(id) => format!("v{}_{}", id, name.label),
        IdentSource::Fn(_) => {
            let fn_ = analysis.fn_(name).expect("internal error: missing fn");
            if let Some(gpu) = &fn_.ast.gpu_qualifier {
                if let Some(source_type) = &fn_.source_type {
                    to_type_wgsl(analysis, source_type)
                } else if let Some(name) = &gpu.name {
                    to_gpu_name_wgsl(analysis, name)
                } else {
                    fn_.ast.name.label.clone()
                }
            } else {
                format!("f{}_{}", fn_.ast.name.id, fn_.ast.name.label)
            }
        }
        IdentSource::Field => {
            let type_ = &analysis.types[ident
                .type_id
                .as_ref()
                .expect("internal error: missing type")];
            if type_
                .ast
                .as_ref()
                .map_or(true, |ast| ast.gpu_qualifier.is_some())
            {
                name.label.clone()
            } else {
                format!("s_{}", name.label)
            }
        }
        IdentSource::GenericType => {
            let type_id = ident
                .type_id
                .as_ref()
                .expect("internal error: missing type");
            to_type_wgsl(analysis, type_id)
        }
    }
}

pub(crate) fn to_buffer_ident_wgsl(analysis: &Analysis, buffer: &BufferId) -> String {
    let name = &analysis.buffers[buffer].ast.name;
    format!("b{}_{}", name.id, name.label)
}

pub(crate) fn to_type_wgsl(analysis: &Analysis, type_id: &TypeId) -> String {
    let type_ = &analysis.types[type_id];
    if let Some(type_) = &type_.ast {
        if let Some(gpu) = &type_.gpu_qualifier {
            if let Some(name) = &gpu.name {
                to_gpu_name_wgsl(analysis, name)
            } else {
                type_.name.label.clone()
            }
        } else {
            format!("t{}_{}", type_.name.id, type_.name.label)
        }
    } else if type_.id == TypeId::from_builtin("bool") {
        "u32".into()
    } else {
        type_.name.clone()
    }
}

fn to_gpu_name_wgsl(analysis: &Analysis, name: &AstGpuName) -> String {
    if name.generics.is_empty() {
        name.root.label.clone()
    } else {
        format!(
            "{}<{}>",
            name.root.label,
            name.generics
                .iter()
                .map(|param| match param {
                    AstGpuGenericParam::Ident(ident) => to_ident_wgsl(analysis, ident),
                    AstGpuGenericParam::Literal(literal) => literal.cleaned_value.clone(),
                })
                .join(", ")
        )
    }
}

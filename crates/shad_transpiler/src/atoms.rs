use crate::{fn_calls, wgsl};
use itertools::Itertools;
use shad_analyzer::{Analysis, BufferId, Function, Item, TypeId};
use shad_parser::{AstExpr, AstExprRoot, AstGpuGenericParam, AstGpuName, AstIdent, AstLiteralType};
use std::iter;

pub(crate) fn to_expr_wgsl(analysis: &Analysis, expr: &AstExpr) -> String {
    let root = match &expr.root {
        AstExprRoot::Ident(ident) => to_var_ident_wgsl(analysis, ident),
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
    let fields = expr.fields.iter().map(to_struct_field_wgsl);
    iter::once(root).chain(fields).join(".")
}

pub(crate) fn to_var_ident_wgsl(analysis: &Analysis, name: &AstIdent) -> String {
    match analysis.item(name) {
        Some(Item::Constant(_)) => unreachable!("internal error: not inlined constant"),
        Some(Item::Buffer(buffer)) => to_buffer_ident_wgsl(analysis, &buffer.id),
        Some(Item::Var(_, _)) | None => format!("v_{}", name.label),
    }
}

pub(crate) fn to_fn_ident_wgsl(analysis: &Analysis, fn_: &Function) -> String {
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

pub(crate) fn to_buffer_ident_wgsl(analysis: &Analysis, buffer: &BufferId) -> String {
    let name = &analysis.buffers[buffer].ast.name;
    format!("b{}_{}", name.id, name.label)
}

pub(crate) fn to_type_wgsl(analysis: &Analysis, type_id: &TypeId) -> String {
    let type_ = &analysis.types[type_id];
    if let Some(type_ast) = &type_.ast {
        if let Some(gpu) = &type_ast.gpu_qualifier {
            if let Some(name) = &gpu.name {
                to_gpu_name_wgsl(analysis, name)
            } else {
                type_ast.name.label.clone()
            }
        } else {
            format!("t_{type_id}")
        }
    } else if type_.id == "bool" {
        "u32".into()
    } else {
        type_.name.clone()
    }
}

pub(crate) fn to_struct_field_wgsl(ident: &AstIdent) -> String {
    if wgsl::is_ident_name_accepted(&ident.label) {
        ident.label.clone()
    } else {
        format!("s_{}", ident.label)
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
                    AstGpuGenericParam::Ident(ident) => {
                        let type_id = analysis
                            .type_id(ident)
                            .expect("internal error: missing type");
                        to_type_wgsl(analysis, &type_id)
                    }
                    AstGpuGenericParam::Literal(literal) => literal.cleaned_value.clone(),
                })
                .join(", ")
        )
    }
}

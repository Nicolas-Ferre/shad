use crate::{atoms, statements};
use itertools::Itertools;
use shad_analyzer::{
    Analysis, BufferId, ComputeShader, FnId, FnParam, Function, StructField, TypeId,
};

pub(crate) fn to_buffer_wgsl(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .buffer_ids
        .iter()
        .enumerate()
        .map(|(index, buffer)| buf_item(analysis, buffer, index))
        .join("\n")
}

pub(crate) fn to_struct_wgsl(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .type_ids
        .iter()
        .filter(|id| {
            analysis.types[id]
                .ast
                .as_ref()
                .map_or(false, |ast| ast.gpu_qualifier.is_none())
        })
        .map(|id| struct_item(analysis, id))
        .join("\n")
}

pub(crate) fn to_fn_wgsl(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .fn_ids
        .iter()
        .filter(|id| {
            let fn_ = &analysis.fns[id];
            !fn_.is_inlined && fn_.ast.gpu_qualifier.is_none()
        })
        .map(|id| fn_item(analysis, id))
        .join("\n")
}

fn buf_item(analysis: &Analysis, buffer: &BufferId, binding_index: usize) -> String {
    let type_ = analysis
        .buffer_type(buffer)
        .expect("internal error: invalid buffer type");
    format!(
        "@group(0) @binding({})\nvar<storage, read_write> {}: {};",
        binding_index,
        atoms::to_buffer_ident_wgsl(analysis, buffer),
        atoms::to_type_wgsl(analysis, &type_.id),
    )
}

fn struct_item(analysis: &Analysis, type_id: &TypeId) -> String {
    let fields = analysis.types[type_id]
        .fields
        .iter()
        .map(|field| struct_field(analysis, field))
        .join(", ");
    format!(
        "struct {} {{ {} }}",
        atoms::to_type_wgsl(analysis, type_id),
        fields
    )
}

fn struct_field(analysis: &Analysis, field: &StructField) -> String {
    let field_type = field
        .type_id
        .as_ref()
        .expect("internal error: invalid field type");
    format!(
        "{}: {}",
        atoms::to_ident_wgsl(analysis, &field.name),
        atoms::to_type_wgsl(analysis, field_type)
    )
}

fn fn_item(analysis: &Analysis, fn_id: &FnId) -> String {
    let fn_ = &analysis.fns[fn_id];
    format!(
        "fn {}({}){} {{\n{}\n}}",
        atoms::to_ident_wgsl(analysis, &fn_.ast.name),
        fn_params(analysis, fn_),
        fn_return_type(analysis, fn_),
        statements::to_wgsl(analysis, &fn_.ast.statements)
    )
}

fn fn_params(analysis: &Analysis, fn_: &Function) -> String {
    fn_.params
        .iter()
        .map(|param| fn_param(analysis, param))
        .join(", ")
}

fn fn_param(analysis: &Analysis, param: &FnParam) -> String {
    let type_id = param
        .type_id
        .as_ref()
        .expect("internal error: invalid param type");
    format!(
        "{}: {}",
        atoms::to_ident_wgsl(analysis, &param.name),
        atoms::to_type_wgsl(analysis, type_id)
    )
}

fn fn_return_type(analysis: &Analysis, type_: &Function) -> String {
    if let Some(type_) = type_
        .return_type_id
        .as_ref()
        .filter(|type_id| analysis.types[type_id].size > 0)
    {
        format!(" -> {}", atoms::to_type_wgsl(analysis, type_))
    } else {
        String::new()
    }
}

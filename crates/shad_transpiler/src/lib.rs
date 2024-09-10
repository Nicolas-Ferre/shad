//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{Buffer, ComputeShader, Expr, Statement, Value};

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
pub fn generate_wgsl_compute_shader(shader: &ComputeShader) -> String {
    format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1) fn main() {{\n{}\n}}",
        buffer_definitions(shader),
        statements(shader)
    )
}

fn buffer_definitions(shader: &ComputeShader) -> String {
    shader
        .buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| buffer_definition(buffer, index))
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_definition(buffer: &Buffer, index: usize) -> String {
    format!(
        "@group(0) @binding({}) var<storage, read_write> {}: {};",
        index,
        buffer_name(buffer),
        buffer.type_.final_name
    )
}

fn statements(shader: &ComputeShader) -> String {
    shader
        .statements
        .iter()
        .map(|stmt| statement(stmt, 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn statement(statement: &Statement, indent: usize) -> String {
    match statement {
        Statement::Assignment(assignment) => {
            format!(
                "{empty: >width$}{} = {};",
                value(&assignment.assigned),
                expr(&assignment.value),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

fn value(assigned: &Value) -> String {
    match assigned {
        Value::Buffer(buffer) => buffer_name(buffer),
    }
}

fn expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(literal) => format!("{}({})", literal.type_.final_name, literal.value),
    }
}

fn buffer_name(buffer: &Buffer) -> String {
    format!("{}_{}", buffer.name.label, buffer.index)
}

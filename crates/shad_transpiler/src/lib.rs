//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{Asg, AsgBuffer, AsgComputeShader, AsgExpr, AsgIdent, AsgStatement, AsgValue};

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
pub fn generate_wgsl_compute_shader(asg: &Asg, shader: &AsgComputeShader) -> String {
    format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1) fn main() {{\n{}\n}}",
        buffer_definitions(asg, shader),
        statements(shader)
    )
}

fn buffer_definitions(asg: &Asg, shader: &AsgComputeShader) -> String {
    shader
        .buffers
        .values()
        .enumerate()
        .map(|(index, buffer)| buffer_definition(asg, buffer, index))
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_definition(asg: &Asg, buffer: &AsgBuffer, index: usize) -> String {
    format!(
        "@group(0) @binding({}) var<storage, read_write> {}: {};",
        index,
        buffer_name(buffer),
        buffer.expr.type_(asg).final_name
    )
}

fn statements(shader: &AsgComputeShader) -> String {
    shader
        .statements
        .iter()
        .map(|stmt| statement(stmt, 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn statement(statement: &AsgStatement, indent: usize) -> String {
    match statement {
        AsgStatement::Assignment(assignment) => {
            format!(
                "{empty: >width$}{} = {};",
                value(&assignment.assigned),
                expression(&assignment.value),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

fn value(assigned: &AsgValue) -> String {
    match assigned {
        // coverage: off (unreachable in `shad_runner` crate)
        AsgValue::Invalid => "<invalid>".into(),
        // coverage: on
        AsgValue::Buffer(buffer) => buffer_name(buffer),
    }
}

fn expression(expr: &AsgExpr) -> String {
    match expr {
        // coverage: off (unreachable in `shad_runner` crate)
        AsgExpr::Invalid => "<invalid>".into(),
        // coverage: on
        AsgExpr::Literal(literal) => format!("{}({})", literal.type_.final_name, literal.value),
        AsgExpr::Ident(AsgIdent::Buffer(buffer)) => buffer_name(buffer),
        AsgExpr::FnCall(fn_call) => format!(
            "{}({})",
            fn_call.fn_.name.label,
            fn_call
                .args
                .iter()
                .map(expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn buffer_name(buffer: &AsgBuffer) -> String {
    format!("{}_{}", buffer.name.label, buffer.index)
}

//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{
    Asg, AsgBuffer, AsgComputeShader, AsgExpr, AsgIdent, AsgStatement, AsgVariable,
};

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
        AsgStatement::Var(assignment) => {
            format!(
                "{empty: >width$}var {} = {};",
                variable_name(assignment),
                expression(&assignment.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AsgStatement::Assignment(assignment) => {
            format!(
                "{empty: >width$}{} = {};",
                ident(&assignment.assigned),
                expression(&assignment.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

fn expression(expr: &AsgExpr) -> String {
    match expr {
        // coverage: off (unreachable in `shad_runner` crate)
        AsgExpr::Invalid => "<invalid>".into(),
        // coverage: on
        AsgExpr::Literal(expr) => format!("{}({})", expr.type_.final_name, expr.value),
        AsgExpr::Ident(expr) => ident(expr),
        AsgExpr::FnCall(expr) => format!(
            "{}({})",
            expr.fn_.name.label,
            expr.args
                .iter()
                .map(expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn ident(ident: &AsgIdent) -> String {
    match ident {
        // coverage: off (unreachable in `shad_runner` crate)
        AsgIdent::Invalid => "<invalid>".into(),
        // coverage: on
        AsgIdent::Buffer(buffer) => buffer_name(buffer),
        AsgIdent::Var(variable) => variable_name(variable),
    }
}

fn buffer_name(buffer: &AsgBuffer) -> String {
    format!("{}_{}", buffer.name.label, buffer.index)
}

fn variable_name(variable: &AsgVariable) -> String {
    format!("{}_{}", variable.name.label, variable.index)
}

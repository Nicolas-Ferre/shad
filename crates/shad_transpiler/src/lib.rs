//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{
    Asg, AsgBuffer, AsgComputeShader, AsgExpr, AsgIdent, AsgStatement, AsgVariable, BOOL_TYPE,
};

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
///
/// # Errors
///
/// An error is returned if the input shader definition is invalid.
#[allow(clippy::result_unit_err)]
pub fn generate_wgsl_compute_shader(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1) fn main() {{\n{}\n}}",
        buffer_definitions(asg, shader)?,
        statements(asg, shader)?
    ))
}

fn buffer_definitions(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(shader
        .buffers
        .values()
        .enumerate()
        .map(|(index, buffer)| buffer_definition(asg, buffer, index))
        .collect::<Result<Vec<_>, ()>>()?
        .join("\n"))
}

fn buffer_definition(asg: &Asg, buffer: &AsgBuffer, index: usize) -> Result<String, ()> {
    Ok(format!(
        "@group(0) @binding({}) var<storage, read_write> {}: {};",
        index,
        buffer_name(asg, buffer, false),
        result_ref(&buffer.expr)?.type_(asg)?.buf_final_name
    ))
}

fn statements(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(shader
        .statements
        .iter()
        .map(|stmt| statement(asg, stmt, 1))
        .collect::<Result<Vec<_>, ()>>()?
        .join("\n"))
}

fn statement(asg: &Asg, statement: &AsgStatement, indent: usize) -> Result<String, ()> {
    Ok(match statement {
        AsgStatement::Var(assignment) => {
            format!(
                "{empty: >width$}var {} = {};",
                variable_name(assignment),
                expression(asg, result_ref(&assignment.expr)?, false),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AsgStatement::Assignment(assignment) => {
            format!(
                "{empty: >width$}{} = {};",
                ident(asg, result_ref(&assignment.assigned)?, false),
                expression(
                    asg,
                    result_ref(&assignment.expr)?,
                    matches!(assignment.assigned, Ok(AsgIdent::Buffer(_))),
                ),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    })
}

fn expression(asg: &Asg, expr: &AsgExpr, is_buffer: bool) -> String {
    match expr {
        AsgExpr::Literal(expr) => {
            let type_name = if is_buffer {
                &expr.type_.buf_final_name
            } else {
                &expr.type_.var_final_name
            };
            format!("{}({})", type_name, expr.value)
        }
        AsgExpr::Ident(expr) => ident(asg, expr, true),
        AsgExpr::FnCall(expr) => format!(
            "{}({})",
            expr.fn_.name.label,
            expr.args
                .iter()
                .map(|arg| expression(asg, arg, false))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn ident(asg: &Asg, ident: &AsgIdent, is_expr: bool) -> String {
    match ident {
        AsgIdent::Buffer(buffer) => buffer_name(asg, buffer, is_expr),
        AsgIdent::Var(variable) => variable_name(variable),
    }
}

fn buffer_name(asg: &Asg, buffer: &AsgBuffer, is_expr: bool) -> String {
    let type_name = result_ref(&buffer.expr)
        .and_then(|expr| expr.type_(asg))
        .map(|type_| type_.name.as_str());
    if type_name == Ok(BOOL_TYPE) && is_expr {
        format!("bool({}_{})", buffer.name.label, buffer.index)
    } else {
        format!("{}_{}", buffer.name.label, buffer.index)
    }
}

fn variable_name(variable: &AsgVariable) -> String {
    format!("{}_{}", variable.name.label, variable.index)
}

fn result_ref<T>(result: &Result<T, ()>) -> Result<&T, ()> {
    result.as_ref().map_err(|&()| ())
}

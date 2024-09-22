//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{
    Asg, AsgBuffer, AsgComputeShader, AsgExpr, AsgFnCall, AsgIdent, AsgStatement, AsgVariable,
    ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN,
    NOT_FN, OR_FN, SUB_FN,
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
        buffer_name(buffer),
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
                expression(asg, result_ref(&assignment.expr)?)?,
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AsgStatement::Assignment(assignment) => {
            let is_buffer = matches!(assignment.assigned, Ok(AsgIdent::Buffer(_)));
            let type_ = result_ref(&assignment.expr)?.type_(asg)?;
            let cast = if is_buffer && type_.buf_final_name != type_.expr_final_name {
                &type_.buf_final_name
            } else {
                ""
            };
            format!(
                "{empty: >width$}{} = {cast}({});",
                ident(asg, result_ref(&assignment.assigned)?, false)?,
                expression(asg, result_ref(&assignment.expr)?)?,
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    })
}

fn expression(asg: &Asg, expr: &AsgExpr) -> Result<String, ()> {
    Ok(match expr {
        AsgExpr::Literal(expr) => format!("{}({})", expr.type_.expr_final_name, expr.value),
        AsgExpr::Ident(expr) => ident(asg, expr, true)?,
        AsgExpr::FnCall(expr) => fn_call(asg, expr)?,
    })
}

fn ident(asg: &Asg, ident: &AsgIdent, is_expr: bool) -> Result<String, ()> {
    Ok(match ident {
        AsgIdent::Buffer(buffer) => {
            let type_ = result_ref(&buffer.expr)?.type_(asg)?;
            if is_expr && type_.buf_final_name != type_.expr_final_name {
                format!("{}({})", type_.expr_final_name, buffer_name(buffer))
            } else {
                buffer_name(buffer)
            }
        }
        AsgIdent::Var(variable) => variable_name(variable),
    })
}

fn fn_call(asg: &Asg, expr: &AsgFnCall) -> Result<String, ()> {
    Ok(if let Some(operator) = unary_operator(expr) {
        format!("({}{})", operator, expression(asg, &expr.args[0])?)
    } else if let Some(operator) = binary_operator(expr) {
        format!(
            "({} {} {})",
            expression(asg, &expr.args[0])?,
            operator,
            expression(asg, &expr.args[1])?
        )
    } else {
        format!(
            "{}({})",
            expr.fn_.name.label,
            expr.args
                .iter()
                .map(|arg| expression(asg, arg))
                .collect::<Result<Vec<_>, ()>>()?
                .join(", ")
        )
    })
}

fn unary_operator(expr: &AsgFnCall) -> Option<&str> {
    match expr.fn_.name.label.as_str() {
        n if n == NEG_FN => Some("-"),
        n if n == NOT_FN => Some("!"),
        _ => None,
    }
}

fn binary_operator(expr: &AsgFnCall) -> Option<&str> {
    match expr.fn_.name.label.as_str() {
        n if n == ADD_FN => Some("+"),
        n if n == SUB_FN => Some("-"),
        n if n == MUL_FN => Some("*"),
        n if n == DIV_FN => Some("/"),
        n if n == MOD_FN => Some("%"),
        n if n == EQ_FN => Some("=="),
        n if n == NE_FN => Some("!="),
        n if n == GT_FN => Some(">"),
        n if n == LT_FN => Some("<"),
        n if n == GE_FN => Some(">="),
        n if n == LE_FN => Some("<="),
        n if n == AND_FN => Some("&&"),
        n if n == OR_FN => Some("||"),
        _ => None,
    }
}

fn buffer_name(buffer: &AsgBuffer) -> String {
    format!("{}_{}", buffer.name.label, buffer.index)
}

fn variable_name(variable: &AsgVariable) -> String {
    format!("{}_{}", variable.name.label, variable.index)
}

fn result_ref<T>(result: &Result<T, ()>) -> Result<&T, ()> {
    result.as_ref().map_err(|&()| ())
}

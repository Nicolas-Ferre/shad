//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{
    Asg, AsgBuffer, AsgComputeShader, AsgExpr, AsgFn, AsgFnCall, AsgFnParam, AsgIdent,
    AsgStatement, AsgVariable, ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN,
    MUL_FN, NEG_FN, NE_FN, NOT_FN, OR_FN, SUB_FN,
};
use shad_parser::AstFnQualifier;

const IDENT_UNIT: usize = 4;

// TODO: split this file

/// Generates a WGSL shader from a Shad shader definition.
///
/// # Errors
///
/// An error is returned if the input shader definition is invalid.
#[allow(clippy::result_unit_err)]
pub fn generate_wgsl_compute_shader(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(format!(
        "{}\n\n{}\n\n@compute @workgroup_size(1, 1, 1) fn main() {{\n{}\n}}",
        wgsl_buf_definitions(asg, shader)?,
        wgsl_fn_definitions(asg, shader)?,
        wgsl_statements(asg, &shader.statements)?
    ))
}

fn wgsl_buf_definitions(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(shader
        .buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| wgsl_buf_definition(asg, buffer, index))
        .collect::<Result<Vec<_>, ()>>()?
        .join("\n"))
}

fn wgsl_buf_definition(asg: &Asg, buffer: &AsgBuffer, binding_index: usize) -> Result<String, ()> {
    Ok(format!(
        "@group(0) @binding({}) var<storage, read_write> {}: {};",
        binding_index,
        buf_name(buffer),
        result_ref(&buffer.expr)?.type_(asg)?.buf_final_name
    ))
}

fn wgsl_fn_definitions(asg: &Asg, shader: &AsgComputeShader) -> Result<String, ()> {
    Ok(shader
        .functions
        .iter()
        .map(|fn_| wgsl_fn_definition(asg, fn_))
        .collect::<Result<Vec<_>, ()>>()?
        .join("\n"))
}

fn wgsl_fn_definition(asg: &Asg, fn_: &AsgFn) -> Result<String, ()> {
    Ok(if fn_.ast.qualifier == AstFnQualifier::Gpu {
        String::new()
    } else {
        format!(
            "fn {}({}) -> {} {{\n{}\n}}",
            fn_name(fn_),
            wgsl_fn_params(fn_)?,
            result_ref(&fn_.return_type)?.expr_final_name,
            wgsl_statements(asg, &asg.function_bodies[&fn_.signature].statements)?
        )
    })
}

fn wgsl_fn_params(fn_: &AsgFn) -> Result<String, ()> {
    Ok(fn_
        .params
        .iter()
        .map(|param| wgsl_fn_param(param))
        .collect::<Result<Vec<_>, ()>>()?
        .join(", "))
}

fn wgsl_fn_param(param: &AsgFnParam) -> Result<String, ()> {
    Ok(format!(
        "{}: {}",
        fn_param_name(param),
        result_ref(&param.type_)?.expr_final_name
    ))
}

fn wgsl_statements(asg: &Asg, statements: &[AsgStatement]) -> Result<String, ()> {
    Ok(statements
        .iter()
        .map(|statement| wgsl_statement(asg, statement, 1))
        .collect::<Result<Vec<_>, ()>>()?
        .join("\n"))
}

fn wgsl_statement(asg: &Asg, statement: &AsgStatement, indent: usize) -> Result<String, ()> {
    Ok(match statement {
        AsgStatement::Var(assignment) => {
            format!(
                "{empty: >width$}var {} = {};",
                var_name(assignment),
                wgsl_expr(asg, result_ref(&assignment.expr)?)?,
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
                wgsl_ident(asg, result_ref(&assignment.assigned)?, false)?,
                wgsl_expr(asg, result_ref(&assignment.expr)?)?,
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AsgStatement::Return(expr) => format!("return {};", wgsl_expr(asg, result_ref(expr)?)?,),
    })
}

fn wgsl_expr(asg: &Asg, expr: &AsgExpr) -> Result<String, ()> {
    Ok(match expr {
        AsgExpr::Literal(expr) => format!("{}({})", expr.type_.expr_final_name, expr.value),
        AsgExpr::Ident(expr) => wgsl_ident(asg, expr, true)?,
        AsgExpr::FnCall(expr) => wgsl_fn_call(asg, expr)?,
    })
}

fn wgsl_ident(asg: &Asg, ident: &AsgIdent, is_expr: bool) -> Result<String, ()> {
    Ok(match ident {
        AsgIdent::Buffer(buffer) => {
            let type_ = result_ref(&buffer.expr)?.type_(asg)?;
            if is_expr && type_.buf_final_name != type_.expr_final_name {
                format!("{}({})", type_.expr_final_name, buf_name(buffer))
            } else {
                buf_name(buffer)
            }
        }
        AsgIdent::Var(variable) => var_name(variable),
        AsgIdent::Param(param) => fn_param_name(param),
    })
}

fn wgsl_fn_call(asg: &Asg, call: &AsgFnCall) -> Result<String, ()> {
    Ok(if let Some(operator) = wgsl_unary_operator(call) {
        format!("({}{})", operator, wgsl_expr(asg, &call.args[0])?)
    } else if let Some(operator) = wgsl_binary_operator(call) {
        format!(
            "({} {} {})",
            wgsl_expr(asg, &call.args[0])?,
            operator,
            wgsl_expr(asg, &call.args[1])?
        )
    } else {
        format!(
            "{}({})",
            fn_name(&call.fn_),
            call.args
                .iter()
                .map(|arg| wgsl_expr(asg, arg))
                .collect::<Result<Vec<_>, ()>>()?
                .join(", ")
        )
    })
}

fn wgsl_unary_operator(expr: &AsgFnCall) -> Option<&str> {
    if expr.fn_.ast.qualifier == AstFnQualifier::Gpu {
        match expr.fn_.name.label.as_str() {
            n if n == NEG_FN => Some("-"),
            n if n == NOT_FN => Some("!"),
            _ => None,
        }
    } else {
        None
    }
}

fn wgsl_binary_operator(expr: &AsgFnCall) -> Option<&str> {
    if expr.fn_.ast.qualifier == AstFnQualifier::Gpu {
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
    } else {
        None
    }
}

fn buf_name(buffer: &AsgBuffer) -> String {
    format!("b{}_{}", buffer.index, buffer.name.label)
}

fn fn_name(fn_: &AsgFn) -> String {
    if fn_.ast.qualifier == AstFnQualifier::Gpu {
        fn_.name.label.clone()
    } else {
        format!("f{}_{}", fn_.index, fn_.name.label)
    }
}

fn fn_param_name(param: &AsgFnParam) -> String {
    format!("p{}", param.name.label)
}

fn var_name(variable: &AsgVariable) -> String {
    format!("v{}_{}", variable.index, variable.name.label)
}

fn result_ref<T>(result: &Result<T, ()>) -> Result<&T, ()> {
    result.as_ref().map_err(|&()| ())
}

//! Transpiler to convert Shad expressions to WGSL.

use itertools::Itertools;
use shad_analyzer::{Analysis, BufferId, ComputeShader, FnId, Function, IdentSource};
use shad_parser::{AstExpr, AstFnCall, AstFnQualifier, AstIdent, AstLeftValue, AstStatement, ADD_FN, AND_FN, DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN, NOT_FN, OR_FN, SUB_FN, AstFnParam};

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
///
/// # Errors
///
/// An error is returned if the input shader definition is invalid.
#[allow(clippy::result_unit_err)]
pub fn generate_wgsl_compute_shader(analysis: &Analysis, shader: &ComputeShader) -> String {
    format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n}}\n\n{}",
        wgsl_buf_definitions(analysis, shader),
        wgsl_statements(analysis, &shader.statements),
        wgsl_fn_definitions(analysis, shader),
    )
}

fn wgsl_buf_definitions(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| wgsl_buf_definition(analysis, buffer, index))
        .join("\n")
}

fn wgsl_buf_definition(analysis: &Analysis, buffer: &BufferId, binding_index: usize) -> String {
    format!(
        "@group(0) @binding({})\nvar<storage, read_write> {}: {};",
        binding_index,
        buf_name(analysis, buffer),
        analysis.buffer_type(buffer).buffer_name,
    )
}

fn wgsl_fn_definitions(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .fn_ids
        .iter()
        .filter(|id| {
            let fn_ = &analysis.fns[id];
            !fn_.is_inlined && analysis.fns[id].ast.qualifier != AstFnQualifier::Gpu
        })
        .map(|id| wgsl_fn_definition(analysis, id))
        .join("\n")
}

fn wgsl_fn_definition(analysis: &Analysis, fn_id: &FnId) -> String {
    let fn_ = &analysis.fns[fn_id];
    format!(
        "fn {}({}){} {{\n{}\n}}",
        fn_name(analysis, &fn_.ast.name),
        wgsl_fn_params(analysis, fn_),
        wgsl_return_type(fn_),
        wgsl_statements(analysis, &fn_.ast.statements)
    )
}

fn wgsl_fn_params(analysis: &Analysis, fn_: &Function) -> String {
    fn_.ast
        .params
        .iter()
        .map(|param| wgsl_fn_param(analysis, param))
        .join(", ")
}

fn wgsl_fn_param(analysis: &Analysis, param: &AstFnParam) -> String {
    format!(
        "{}: {}",
        wgsl_ident(analysis, &param.name, false),
        param.type_.label
    )
}

fn wgsl_return_type(type_: &Function) -> String {
    if let Some(type_) = &type_.ast.return_type {
        format!(" -> {}", type_.name.label)
    } else {
        String::new()
    }
}

fn wgsl_statements(analysis: &Analysis, statements: &[AstStatement]) -> String {
    statements
        .iter()
        .map(|statement| wgsl_statement(analysis, statement, 1))
        .join("\n")
}

fn wgsl_statement(analysis: &Analysis, statement: &AstStatement, indent: usize) -> String {
    match statement {
        AstStatement::Var(statement) => {
            format!(
                "{empty: >width$}var {} = {};",
                wgsl_ident(analysis, &statement.name, false),
                wgsl_expr(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Assignment(statement) => match &statement.value {
            AstLeftValue::Ident(assigned) => {
                let value_ident = &analysis.idents[&assigned.id];
                let is_buffer = matches!(value_ident.source, IdentSource::Buffer(_));
                let type_name = value_ident
                    .type_
                    .as_ref()
                    .expect("internal error: missing type");
                let type_ = &analysis.types[type_name];
                let cast = if is_buffer && type_.buffer_name != type_.name {
                    &type_.buffer_name
                } else {
                    ""
                };
                format!(
                    "{empty: >width$}{} = {cast}({});",
                    wgsl_ident(analysis, assigned, false),
                    wgsl_expr(analysis, &statement.expr),
                    empty = "",
                    width = indent * IDENT_UNIT,
                )
            }
            AstLeftValue::FnCall(_) => unreachable!("internal error: invalid inlining"),
        },
        AstStatement::Return(statement) => {
            format!("return {};", wgsl_expr(analysis, &statement.expr))
        }
        AstStatement::FnCall(statement) => {
            format!("{};", wgsl_fn_call(analysis, &statement.call))
        }
    }
}

fn wgsl_expr(analysis: &Analysis, expr: &AstExpr) -> String {
    match expr {
        AstExpr::Literal(expr) => expr.value.clone(),
        AstExpr::Ident(expr) => wgsl_ident(analysis, expr, true),
        AstExpr::FnCall(expr) => wgsl_fn_call(analysis, expr),
    }
}

fn wgsl_ident(analysis: &Analysis, name: &AstIdent, is_expr: bool) -> String {
    match &analysis.idents[&name.id].source {
        IdentSource::Buffer(name) => {
            let type_ = analysis.buffer_type(name);
            if is_expr && type_.buffer_name != type_.name {
                format!("{}({})", type_.name, buf_name(analysis, name))
            } else {
                buf_name(analysis, name)
            }
        }
        IdentSource::Var(id) => format!("v{}_{}", id, name.label),
        IdentSource::Fn(_) => unreachable!("internal error: variable as function"),
    }
}

fn wgsl_fn_call(analysis: &Analysis, call: &AstFnCall) -> String {
    if let Some(operator) = wgsl_unary_operator(analysis, call) {
        format!("({}{})", operator, wgsl_expr(analysis, &call.args[0]))
    } else if let Some(operator) = wgsl_binary_operator(analysis, call) {
        format!(
            "({} {} {})",
            wgsl_expr(analysis, &call.args[0]),
            operator,
            wgsl_expr(analysis, &call.args[1])
        )
    } else {
        format!(
            "{}({})",
            fn_name(analysis, &call.name),
            call.args
                .iter()
                .map(|arg| wgsl_expr(analysis, arg))
                .join(", ")
        )
    }
}

fn wgsl_unary_operator(analysis: &Analysis, call: &AstFnCall) -> Option<&'static str> {
    let fn_ = fn_(analysis, &call.name);
    if fn_.ast.qualifier == AstFnQualifier::Gpu {
        match fn_.ast.name.label.as_str() {
            n if n == NEG_FN => Some("-"),
            n if n == NOT_FN => Some("!"),
            _ => None,
        }
    } else {
        None
    }
}

fn wgsl_binary_operator(analysis: &Analysis, call: &AstFnCall) -> Option<&'static str> {
    let fn_ = fn_(analysis, &call.name);
    if fn_.ast.qualifier == AstFnQualifier::Gpu {
        match fn_.ast.name.label.as_str() {
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

fn buf_name(analysis: &Analysis, buffer: &BufferId) -> String {
    let name = &analysis.buffers[buffer].ast.name;
    format!("b{}_{}", name.id, name.label)
}

fn fn_name(analysis: &Analysis, ident: &AstIdent) -> String {
    let fn_ = fn_(analysis, ident);
    if fn_.ast.qualifier == AstFnQualifier::Gpu {
        fn_.ast.name.label.clone()
    } else {
        format!("f{}_{}", fn_.ast.name.id, fn_.ast.name.label)
    }
}

fn fn_<'a>(analysis: &'a Analysis, ident: &AstIdent) -> &'a Function {
    match &analysis.idents[&ident.id].source {
        IdentSource::Fn(signature) => &analysis.fns[signature],
        IdentSource::Buffer(_) | IdentSource::Var(_) => {
            unreachable!("internal error: invalid fn")
        }
    }
}

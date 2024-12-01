//! Transpiler to convert Shad expressions to WGSL.

use itertools::Itertools;
use shad_analyzer::{
    Analysis, BufferId, ComputeShader, FnId, FnParam, Function, IdentSource, StructField, TypeId,
};
use shad_parser::{
    AstExpr, AstFnCall, AstFnQualifier, AstIdent, AstLeftValue, AstStatement, ADD_FN, AND_FN,
    DIV_FN, EQ_FN, GE_FN, GT_FN, LE_FN, LT_FN, MOD_FN, MUL_FN, NEG_FN, NE_FN, NOT_FN, OR_FN,
    SUB_FN,
};

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
///
/// # Errors
///
/// An error is returned if the input shader definition is invalid.
#[allow(clippy::result_unit_err)]
pub fn generate_wgsl_compute_shader(analysis: &Analysis, shader: &ComputeShader) -> String {
    format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n}}\n\n{}\n\n{}",
        wgsl_buffer_items(analysis, shader),
        wgsl_statements(analysis, &shader.statements),
        wgsl_struct_items(analysis, shader),
        wgsl_fn_items(analysis, shader),
    )
}

fn wgsl_buffer_items(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .buffer_ids
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
        type_name(analysis, &analysis.buffer_type(buffer).id),
    )
}

fn wgsl_struct_items(analysis: &Analysis, shader: &ComputeShader) -> String {
    shader
        .type_ids
        .iter()
        .map(|type_| wgsl_type_definition(analysis, type_))
        .join("\n")
}

fn wgsl_type_definition(analysis: &Analysis, type_id: &TypeId) -> String {
    let type_ = &analysis.types[type_id];
    if type_.ast.is_some() {
        let fields = type_
            .fields
            .iter()
            .map(|field| wgsl_type_field(analysis, field))
            .join(", ");
        format!("struct {} {{ {} }}", type_name(analysis, type_id), fields)
    } else {
        String::new()
    }
}

fn wgsl_type_field(analysis: &Analysis, field: &StructField) -> String {
    let field_type = field
        .type_id
        .as_ref()
        .expect("internal error: invalid field type");
    format!(
        "{}: {}",
        field_name(&field.name),
        type_name(analysis, field_type)
    )
}

fn wgsl_fn_items(analysis: &Analysis, shader: &ComputeShader) -> String {
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
        wgsl_return_type(analysis, fn_),
        wgsl_statements(analysis, &fn_.ast.statements)
    )
}

fn wgsl_fn_params(analysis: &Analysis, fn_: &Function) -> String {
    fn_.params
        .iter()
        .map(|param| wgsl_fn_param(analysis, param))
        .join(", ")
}

fn wgsl_fn_param(analysis: &Analysis, param: &FnParam) -> String {
    let type_id = param
        .type_id
        .as_ref()
        .expect("internal error: invalid param type");
    format!(
        "{}: {}",
        wgsl_ident(analysis, &param.name),
        type_name(analysis, type_id)
    )
}

fn wgsl_return_type(analysis: &Analysis, type_: &Function) -> String {
    if let Some(type_) = &type_.return_type_id {
        format!(" -> {}", type_name(analysis, type_))
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
                wgsl_ident(analysis, &statement.name),
                wgsl_expr(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::Assignment(statement) => match &statement.value {
            AstLeftValue::Ident(assigned) => {
                format!(
                    "{empty: >width$}{} = {};",
                    wgsl_ident(analysis, assigned),
                    wgsl_expr(analysis, &statement.expr),
                    empty = "",
                    width = indent * IDENT_UNIT,
                )
            }
            AstLeftValue::FnCall(_) => unreachable!("internal error: invalid inlining"),
        },
        AstStatement::Return(statement) => {
            format!(
                "{empty: >width$}return {};",
                wgsl_expr(analysis, &statement.expr),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
        AstStatement::FnCall(statement) => {
            format!(
                "{empty: >width$}{};",
                wgsl_fn_call(analysis, &statement.call),
                empty = "",
                width = indent * IDENT_UNIT,
            )
        }
    }
}

fn wgsl_expr(analysis: &Analysis, expr: &AstExpr) -> String {
    match expr {
        AstExpr::Literal(expr) => match expr.value.as_str() {
            "false" => "0u".into(),
            "true" => "1u".into(),
            _ => expr.value.clone(),
        },
        AstExpr::Ident(expr) => wgsl_ident(analysis, expr),
        AstExpr::FnCall(expr) => wgsl_fn_call(analysis, expr),
    }
}

fn wgsl_ident(analysis: &Analysis, name: &AstIdent) -> String {
    match &analysis.idents[&name.id].source {
        IdentSource::Buffer(name) => buf_name(analysis, name),
        IdentSource::Var(id) => format!("v{}_{}", id, name.label),
        IdentSource::Fn(_) => unreachable!("internal error: variable as function"),
    }
}

fn wgsl_fn_call(analysis: &Analysis, call: &AstFnCall) -> String {
    let fn_ = fn_(analysis, &call.name);
    wgsl_cast_fn_call(
        fn_,
        if let Some(operator) = wgsl_unary_operator(analysis, call) {
            format!(
                "({}{})",
                operator,
                wgsl_cast_fn_arg(analysis, fn_, &fn_.params[0], &call.args[0])
            )
        } else if let Some(operator) = wgsl_binary_operator(analysis, call) {
            format!(
                "({} {} {})",
                wgsl_cast_fn_arg(analysis, fn_, &fn_.params[0], &call.args[0]),
                operator,
                wgsl_cast_fn_arg(analysis, fn_, &fn_.params[1], &call.args[1])
            )
        } else {
            format!(
                "{}({})",
                fn_name(analysis, &call.name),
                call.args
                    .iter()
                    .zip(&fn_.params)
                    .map(|(arg, param)| wgsl_cast_fn_arg(analysis, fn_, param, arg))
                    .join(", ")
            )
        },
    )
}

fn wgsl_cast_fn_call(fn_: &Function, call: String) -> String {
    if let Some(return_type_id) = &fn_.return_type_id {
        if return_type_id == &TypeId::from_builtin("bool") {
            format!("u32({call})")
        } else {
            call
        }
    } else {
        call
    }
}

fn wgsl_cast_fn_arg(analysis: &Analysis, fn_: &Function, param: &FnParam, arg: &AstExpr) -> String {
    let expr = wgsl_expr(analysis, arg);
    let type_ = &analysis.types[param
        .type_id
        .as_ref()
        .expect("internal error: invalid param type")];
    if fn_.ast.qualifier == AstFnQualifier::Gpu
        && fn_.source_type.is_none()
        && type_.id == TypeId::from_builtin("bool")
    {
        format!("bool({expr})")
    } else {
        expr
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

fn field_name(name: &AstIdent) -> String {
    format!("f{}", name.label)
}

fn type_name(analysis: &Analysis, type_id: &TypeId) -> String {
    let type_ = &analysis.types[type_id];
    if let Some(type_) = &type_.ast {
        format!("t{}_{}", type_.name.id, type_.name.label)
    } else if type_.id == TypeId::from_builtin("bool") {
        "u32".into()
    } else {
        type_.name.clone()
    }
}

fn fn_name(analysis: &Analysis, ident: &AstIdent) -> String {
    let fn_ = fn_(analysis, ident);
    if fn_.ast.qualifier == AstFnQualifier::Gpu {
        if let Some(source_type) = &fn_.source_type {
            type_name(analysis, source_type)
        } else {
            fn_.ast.name.label.clone()
        }
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

use crate::compilation::constant::{ConstantData, ConstantValue};
use crate::language::constants;

pub(crate) fn runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    constructor_runner(fn_key).or_else(|| operator_runner(fn_key))
}

pub(crate) fn constructor_runner(
    fn_key: &str,
) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`bool(f32)` function" | "`bool(i32)` function" | "`bool(u32)` function" => {
            |p| to_bool(p[0]).data
        }
        "`boolx2()` function" => |_| constants::vec2(bool(false), bool(false)),
        "`boolx2(bool, bool)` function" => |p| constants::vec2(p[0].clone(), p[1].clone()),
        "`boolx2(f32x2)` function" | "`boolx2(i32x2)` function" | "`boolx2(u32x2)` function" => {
            |p| {
                let fields = p[0].data.fields();
                constants::vec2(to_bool(&fields[0].value), to_bool(&fields[1].value))
            }
        }
        "`boolx3()` function" => |_| constants::vec3(bool(false), bool(false), bool(false)),
        "`boolx3(bool, bool, bool)` function" => {
            |p| constants::vec3(p[0].clone(), p[1].clone(), p[2].clone())
        }
        "`boolx3(boolx2, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec3(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`boolx3(f32x3)` function" | "`boolx3(i32x3)` function" | "`boolx3(u32x3)` function" => {
            |p| {
                let fields = p[0].data.fields();
                constants::vec3(
                    to_bool(&fields[0].value),
                    to_bool(&fields[1].value),
                    to_bool(&fields[2].value),
                )
            }
        }
        "`boolx4()` function" => {
            |_| constants::vec4(bool(false), bool(false), bool(false), bool(false))
        }
        "`boolx4(bool, bool, bool, bool)` function" => {
            |p| constants::vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`boolx4(boolx2, bool, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`boolx4(boolx3, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                to_bool(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`boolx4(f32x4)` function" | "`boolx4(i32x4)` function" | "`boolx4(u32x4)` function" => {
            |p| {
                let fields = p[0].data.fields();
                constants::vec4(
                    to_bool(&fields[0].value),
                    to_bool(&fields[1].value),
                    to_bool(&fields[2].value),
                    to_bool(&fields[3].value),
                )
            }
        }
        _ => None?,
    })
}

pub(crate) fn operator_runner(
    fn_key: &str,
) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`__lt__(bool, bool)` function"
        | "`__lt__(boolx2, boolx2)` function"
        | "`__lt__(boolx3, boolx3)` function"
        | "`__lt__(boolx4, boolx4)` function" => |p| constants::lt(p[0], p[1]),
        "`__gt__(bool, bool)` function"
        | "`__gt__(boolx2, boolx2)` function"
        | "`__gt__(boolx3, boolx3)` function"
        | "`__gt__(boolx4, boolx4)` function" => |p| constants::gt(p[0], p[1]),
        "`__le__(bool, bool)` function"
        | "`__le__(boolx2, boolx2)` function"
        | "`__le__(boolx3, boolx3)` function"
        | "`__le__(boolx4, boolx4)` function" => |p| constants::le(p[0], p[1]),
        "`__ge__(bool, bool)` function"
        | "`__ge__(boolx2, boolx2)` function"
        | "`__ge__(boolx3, boolx3)` function"
        | "`__ge__(boolx4, boolx4)` function" => |p| constants::ge(p[0], p[1]),
        "`__eq__(bool, bool)` function"
        | "`__eq__(boolx2, boolx2)` function"
        | "`__eq__(boolx3, boolx3)` function"
        | "`__eq__(boolx4, boolx4)` function" => |p| constants::eq(p[0], p[1]),
        "`__ne__(bool, bool)` function"
        | "`__ne__(boolx2, boolx2)` function"
        | "`__ne__(boolx3, boolx3)` function"
        | "`__ne__(boolx4, boolx4)` function" => |p| constants::ne(p[0], p[1]),
        "`__and__(bool, bool)` function" => |p| constants::and(p[0], p[1]),
        "`__or__(bool, bool)` function" => |p| constants::or(p[0], p[1]),
        "`__not__(bool)` function"
        | "`__not__(boolx2)` function"
        | "`__not__(boolx3)` function"
        | "`__not__(boolx4)` function" => |p| constants::not(p[0]),
        _ => None?,
    })
}

fn bool(value: bool) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "u32".into(),
        data: ConstantData::Bool(value),
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
fn to_bool(value: &ConstantValue) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "u32".into(),
        data: match value.data {
            ConstantData::Bool(value) => ConstantData::Bool(value),
            ConstantData::F32(value) => ConstantData::Bool(value != 0.),
            ConstantData::I32(value) => ConstantData::Bool(value != 0),
            ConstantData::U32(value) => ConstantData::Bool(value != 0),
            ConstantData::StructFields(_) => unreachable!("unsupported native conversion"),
        },
    }
}

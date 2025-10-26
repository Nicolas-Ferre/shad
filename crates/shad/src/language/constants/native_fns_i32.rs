use crate::compilation::constant::{ConstantData, ConstantValue};
use crate::language::constants;

pub(crate) fn runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    constructor_runner(fn_key).or_else(|| operator_runner(fn_key))
}

fn constructor_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`i32(bool)` function" | "`i32(f32)` function" | "`i32(u32)` function" => {
            |p| to_i32(p[0]).data
        }
        "`i32x2()` function" => |_| constants::vec2(i32(0), i32(0)),
        "`i32x2(i32, i32)` function" => |p| constants::vec2(p[0].clone(), p[1].clone()),
        "`i32x2(boolx2)` function" | "`i32x2(f32x2)` function" | "`i32x2(u32x2)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec2(to_i32(&fields[0].value), to_i32(&fields[1].value))
        },
        "`i32x3()` function" => |_| constants::vec3(i32(0), i32(0), i32(0)),
        "`i32x3(i32, i32, i32)` function" => {
            |p| constants::vec3(p[0].clone(), p[1].clone(), p[2].clone())
        }
        "`i32x3(i32x2, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec3(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`i32x3(boolx3)` function" | "`i32x3(f32x3)` function" | "`i32x3(u32x3)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec3(
                to_i32(&fields[0].value),
                to_i32(&fields[1].value),
                to_i32(&fields[2].value),
            )
        },
        "`i32x4()` function" => |_| constants::vec4(i32(0), i32(0), i32(0), i32(0)),
        "`i32x4(i32, i32, i32, i32)` function" => {
            |p| constants::vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`i32x4(i32x2, i32, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`i32x4(i32x3, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                to_i32(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`i32x4(boolx4)` function" | "`i32x4(f32x4)` function" | "`i32x4(u32x4)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec4(
                to_i32(&fields[0].value),
                to_i32(&fields[1].value),
                to_i32(&fields[2].value),
                to_i32(&fields[3].value),
            )
        },
        _ => None?,
    })
}

fn operator_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`__add__(i32, i32)` function"
        | "`__add__(i32x2, i32x2)` function"
        | "`__add__(i32x3, i32x3)` function"
        | "`__add__(i32x4, i32x4)` function" => |p| constants::add(p[0], p[1]),
        "`__sub__(i32, i32)` function"
        | "`__sub__(i32x2, i32x2)` function"
        | "`__sub__(i32x3, i32x3)` function"
        | "`__sub__(i32x4, i32x4)` function" => |p| constants::sub(p[0], p[1]),
        "`__mul__(i32, i32)` function"
        | "`__mul__(i32x2, i32x2)` function"
        | "`__mul__(i32x3, i32x3)` function"
        | "`__mul__(i32x4, i32x4)` function" => |p| constants::mul(p[0], p[1]),
        "`__div__(i32, i32)` function"
        | "`__div__(i32x2, i32x2)` function"
        | "`__div__(i32x3, i32x3)` function"
        | "`__div__(i32x4, i32x4)` function" => |p| constants::div(p[0], p[1]),
        "`__mod__(i32, i32)` function"
        | "`__mod__(i32x2, i32x2)` function"
        | "`__mod__(i32x3, i32x3)` function"
        | "`__mod__(i32x4, i32x4)` function" => |p| constants::mod_(p[0], p[1]),
        "`__lt__(i32, i32)` function"
        | "`__lt__(i32x2, i32x2)` function"
        | "`__lt__(i32x3, i32x3)` function"
        | "`__lt__(i32x4, i32x4)` function" => |p| constants::lt(p[0], p[1]),
        "`__gt__(i32, i32)` function"
        | "`__gt__(i32x2, i32x2)` function"
        | "`__gt__(i32x3, i32x3)` function"
        | "`__gt__(i32x4, i32x4)` function" => |p| constants::gt(p[0], p[1]),
        "`__le__(i32, i32)` function"
        | "`__le__(i32x2, i32x2)` function"
        | "`__le__(i32x3, i32x3)` function"
        | "`__le__(i32x4, i32x4)` function" => |p| constants::le(p[0], p[1]),
        "`__ge__(i32, i32)` function"
        | "`__ge__(i32x2, i32x2)` function"
        | "`__ge__(i32x3, i32x3)` function"
        | "`__ge__(i32x4, i32x4)` function" => |p| constants::ge(p[0], p[1]),
        "`__eq__(i32, i32)` function"
        | "`__eq__(i32x2, i32x2)` function"
        | "`__eq__(i32x3, i32x3)` function"
        | "`__eq__(i32x4, i32x4)` function" => |p| constants::eq(p[0], p[1]),
        "`__ne__(i32, i32)` function"
        | "`__ne__(i32x2, i32x2)` function"
        | "`__ne__(i32x3, i32x3)` function"
        | "`__ne__(i32x4, i32x4)` function" => |p| constants::ne(p[0], p[1]),
        "`__neg__(i32)` function"
        | "`__neg__(i32x2)` function"
        | "`__neg__(i32x3)` function"
        | "`__neg__(i32x4)` function" => |p| constants::neg(p[0]),
        _ => None?,
    })
}

fn i32(value: i32) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "i32".into(),
        data: ConstantData::I32(value),
    }
}

#[allow(
    clippy::wildcard_enum_match_arm,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn to_i32(value: &ConstantValue) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "i32".into(),
        data: match value.data {
            ConstantData::Bool(value) => ConstantData::I32(value.into()),
            ConstantData::F32(value) => ConstantData::I32(value as i32),
            ConstantData::I32(value) => ConstantData::I32(value),
            ConstantData::U32(value) => ConstantData::I32(value as i32),
            ConstantData::StructFields(_) => unreachable!("unsupported native conversion"),
        },
    }
}

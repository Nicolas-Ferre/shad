use crate::compilation::constant::{ConstantData, ConstantValue};
use crate::language::constants;

pub(crate) fn runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    constructor_runner(fn_key).or_else(|| operator_runner(fn_key))
}

fn constructor_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`f32(bool)` function" | "`f32(i32)` function" | "`f32(u32)` function" => {
            |p| to_f32(p[0]).data
        }
        "`f32x2()` function" => |_| constants::vec2(f32(0.), f32(0.)),
        "`f32x2(f32, f32)` function" => |p| constants::vec2(p[0].clone(), p[1].clone()),
        "`f32x2(boolx2)` function" | "`f32x2(i32x2)` function" | "`f32x2(u32x2)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec2(to_f32(&fields[0].value), to_f32(&fields[1].value))
        },
        "`f32x3()` function" => |_| constants::vec3(f32(0.), f32(0.), f32(0.)),
        "`f32x3(f32, f32, f32)` function" => {
            |p| constants::vec3(p[0].clone(), p[1].clone(), p[2].clone())
        }
        "`f32x3(f32x2, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec3(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`f32x3(boolx3)` function" | "`f32x3(i32x3)` function" | "`f32x3(u32x3)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec3(
                to_f32(&fields[0].value),
                to_f32(&fields[1].value),
                to_f32(&fields[2].value),
            )
        },
        "`f32x4()` function" => |_| constants::vec4(f32(0.), f32(0.), f32(0.), f32(0.)),
        "`f32x4(f32, f32, f32, f32)` function" => {
            |p| constants::vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`f32x4(f32x2, f32, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`f32x4(f32x3, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            constants::vec4(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                to_f32(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`f32x4(boolx4)` function" | "`f32x4(i32x4)` function" | "`f32x4(u32x4)` function" => |p| {
            let fields = p[0].data.fields();
            constants::vec4(
                to_f32(&fields[0].value),
                to_f32(&fields[1].value),
                to_f32(&fields[2].value),
                to_f32(&fields[3].value),
            )
        },
        _ => None?,
    })
}

fn operator_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`__add__(f32, f32)` function"
        | "`__add__(f32x2, f32x2)` function"
        | "`__add__(f32x3, f32x3)` function"
        | "`__add__(f32x4, f32x4)` function" => |p| constants::add(p[0], p[1]),
        "`__sub__(f32, f32)` function"
        | "`__sub__(f32x2, f32x2)` function"
        | "`__sub__(f32x3, f32x3)` function"
        | "`__sub__(f32x4, f32x4)` function" => |p| constants::sub(p[0], p[1]),
        "`__mul__(f32, f32)` function"
        | "`__mul__(f32x2, f32x2)` function"
        | "`__mul__(f32x3, f32x3)` function"
        | "`__mul__(f32x4, f32x4)` function" => |p| constants::mul(p[0], p[1]),
        "`__div__(f32, f32)` function"
        | "`__div__(f32x2, f32x2)` function"
        | "`__div__(f32x3, f32x3)` function"
        | "`__div__(f32x4, f32x4)` function" => |p| constants::div(p[0], p[1]),
        "`__lt__(f32, f32)` function"
        | "`__lt__(f32x2, f32x2)` function"
        | "`__lt__(f32x3, f32x3)` function"
        | "`__lt__(f32x4, f32x4)` function" => |p| constants::lt(p[0], p[1]),
        "`__gt__(f32, f32)` function"
        | "`__gt__(f32x2, f32x2)` function"
        | "`__gt__(f32x3, f32x3)` function"
        | "`__gt__(f32x4, f32x4)` function" => |p| constants::gt(p[0], p[1]),
        "`__le__(f32, f32)` function"
        | "`__le__(f32x2, f32x2)` function"
        | "`__le__(f32x3, f32x3)` function"
        | "`__le__(f32x4, f32x4)` function" => |p| constants::le(p[0], p[1]),
        "`__ge__(f32, f32)` function"
        | "`__ge__(f32x2, f32x2)` function"
        | "`__ge__(f32x3, f32x3)` function"
        | "`__ge__(f32x4, f32x4)` function" => |p| constants::ge(p[0], p[1]),
        "`__eq__(f32, f32)` function"
        | "`__eq__(f32x2, f32x2)` function"
        | "`__eq__(f32x3, f32x3)` function"
        | "`__eq__(f32x4, f32x4)` function" => |p| constants::eq(p[0], p[1]),
        "`__ne__(f32, f32)` function"
        | "`__ne__(f32x2, f32x2)` function"
        | "`__ne__(f32x3, f32x3)` function"
        | "`__ne__(f32x4, f32x4)` function" => |p| constants::ne(p[0], p[1]),
        "`__neg__(f32)` function"
        | "`__neg__(f32x2)` function"
        | "`__neg__(f32x3)` function"
        | "`__neg__(f32x4)` function" => |p| constants::neg(p[0]),
        _ => None?,
    })
}

fn f32(value: f32) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "f32".into(),
        data: ConstantData::F32(value),
    }
}

#[allow(clippy::wildcard_enum_match_arm, clippy::cast_precision_loss)]
fn to_f32(value: &ConstantValue) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "f32".into(),
        data: match value.data {
            ConstantData::Bool(value) => ConstantData::F32(value.into()),
            ConstantData::F32(value) => ConstantData::F32(value),
            ConstantData::I32(value) => ConstantData::F32(value as f32),
            ConstantData::U32(value) => ConstantData::F32(value as f32),
            ConstantData::StructFields(_) => unreachable!("unsupported native conversion"),
        },
    }
}

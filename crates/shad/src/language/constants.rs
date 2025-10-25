use crate::compilation::constant::{
    ConstantContext, ConstantData, ConstantStructFieldData, ConstantValue,
};
use crate::compilation::node::Node;
use crate::language::items::fn_;
use std::collections::HashMap;

// TODO: add operators

pub(crate) fn native_fn_runner(
    fn_key: &str,
) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    bool_native_fn_runner(fn_key)
        .or_else(|| f32_native_fn_runner(fn_key))
        .or_else(|| i32_native_fn_runner(fn_key))
        .or_else(|| u32_native_fn_runner(fn_key))
        .or_else(|| i32_native_fn_runner(fn_key))
}

fn bool_native_fn_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`bool(f32)` function" | "`bool(i32)` function" | "`bool(u32)` function" => {
            |p| to_bool(p[0]).data
        }
        "`boolx2()` function" => |_| vec2(bool(false), bool(false)),
        "`boolx2(bool, bool)` function" => |p| vec2(p[0].clone(), p[1].clone()),
        "`boolx2(f32x2)` function" | "`boolx2(i32x2)` function" | "`boolx2(u32x2)` function" => {
            |p| {
                let fields = p[0].data.fields();
                vec2(to_bool(&fields[0].value), to_bool(&fields[1].value))
            }
        }
        "`boolx3()` function" => |_| vec3(bool(false), bool(false), bool(false)),
        "`boolx3(bool, bool, bool)` function" => |p| vec3(p[0].clone(), p[1].clone(), p[2].clone()),
        "`boolx3(boolx2, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec3(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`boolx3(f32x3)` function" | "`boolx3(i32x3)` function" | "`boolx3(u32x3)` function" => {
            |p| {
                let fields = p[0].data.fields();
                vec3(
                    to_bool(&fields[0].value),
                    to_bool(&fields[1].value),
                    to_bool(&fields[2].value),
                )
            }
        }
        "`boolx4()` function" => |_| vec4(bool(false), bool(false), bool(false), bool(false)),
        "`boolx4(bool, bool, bool, bool)` function" => {
            |p| vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`boolx4(boolx2, bool, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`boolx4(boolx3, bool)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_bool(&param1_fields[0].value),
                to_bool(&param1_fields[1].value),
                to_bool(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`boolx4(f32x4)` function" | "`boolx4(i32x4)` function" | "`boolx4(u32x4)` function" => {
            |p| {
                let fields = p[0].data.fields();
                vec4(
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

fn f32_native_fn_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`f32(bool)` function" | "`f32(i32)` function" | "`f32(u32)` function" => {
            |p| to_f32(p[0]).data
        }
        "`f32x2()` function" => |_| vec2(f32(0.), f32(0.)),
        "`f32x2(f32, f32)` function" => |p| vec2(p[0].clone(), p[1].clone()),
        "`f32x2(boolx2)` function" | "`f32x2(i32x2)` function" | "`f32x2(u32x2)` function" => |p| {
            let fields = p[0].data.fields();
            vec2(to_f32(&fields[0].value), to_f32(&fields[1].value))
        },
        "`f32x3()` function" => |_| vec3(f32(0.), f32(0.), f32(0.)),
        "`f32x3(f32, f32, f32)` function" => |p| vec3(p[0].clone(), p[1].clone(), p[2].clone()),
        "`f32x3(f32x2, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec3(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`f32x3(boolx3)` function" | "`f32x3(i32x3)` function" | "`f32x3(u32x3)` function" => |p| {
            let fields = p[0].data.fields();
            vec3(
                to_f32(&fields[0].value),
                to_f32(&fields[1].value),
                to_f32(&fields[2].value),
            )
        },
        "`f32x4()` function" => |_| vec4(f32(0.), f32(0.), f32(0.), f32(0.)),
        "`f32x4(f32, f32, f32, f32)` function" => {
            |p| vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`f32x4(f32x2, f32, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`f32x4(f32x3, f32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_f32(&param1_fields[0].value),
                to_f32(&param1_fields[1].value),
                to_f32(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`f32x4(boolx4)` function" | "`f32x4(i32x4)` function" | "`f32x4(u32x4)` function" => |p| {
            let fields = p[0].data.fields();
            vec4(
                to_f32(&fields[0].value),
                to_f32(&fields[1].value),
                to_f32(&fields[2].value),
                to_f32(&fields[3].value),
            )
        },
        _ => None?,
    })
}

fn i32_native_fn_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`i32(bool)` function" | "`i32(f32)` function" | "`i32(u32)` function" => {
            |p| to_i32(p[0]).data
        }
        "`i32x2()` function" => |_| vec2(i32(0), i32(0)),
        "`i32x2(i32, i32)` function" => |p| vec2(p[0].clone(), p[1].clone()),
        "`i32x2(boolx2)` function" | "`i32x2(f32x2)` function" | "`i32x2(u32x2)` function" => |p| {
            let fields = p[0].data.fields();
            vec2(to_i32(&fields[0].value), to_i32(&fields[1].value))
        },
        "`i32x3()` function" => |_| vec3(i32(0), i32(0), i32(0)),
        "`i32x3(i32, i32, i32)` function" => |p| vec3(p[0].clone(), p[1].clone(), p[2].clone()),
        "`i32x3(i32x2, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec3(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`i32x3(boolx3)` function" | "`i32x3(f32x3)` function" | "`i32x3(u32x3)` function" => |p| {
            let fields = p[0].data.fields();
            vec3(
                to_i32(&fields[0].value),
                to_i32(&fields[1].value),
                to_i32(&fields[2].value),
            )
        },
        "`i32x4()` function" => |_| vec4(i32(0), i32(0), i32(0), i32(0)),
        "`i32x4(i32, i32, i32, i32)` function" => {
            |p| vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`i32x4(i32x2, i32, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`i32x4(i32x3, i32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_i32(&param1_fields[0].value),
                to_i32(&param1_fields[1].value),
                to_i32(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`i32x4(boolx4)` function" | "`i32x4(f32x4)` function" | "`i32x4(u32x4)` function" => |p| {
            let fields = p[0].data.fields();
            vec4(
                to_i32(&fields[0].value),
                to_i32(&fields[1].value),
                to_i32(&fields[2].value),
                to_i32(&fields[3].value),
            )
        },
        _ => None?,
    })
}

fn u32_native_fn_runner(fn_key: &str) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    Some(match fn_key {
        "`u32(bool)` function" | "`u32(f32)` function" | "`u32(i32)` function" => {
            |p| to_u32(p[0]).data
        }
        "`u32x2()` function" => |_| vec2(u32(0), u32(0)),
        "`u32x2(u32, u32)` function" => |p| vec2(p[0].clone(), p[1].clone()),
        "`u32x2(boolx2)` function" | "`u32x2(f32x2)` function" | "`u32x2(i32x2)` function" => |p| {
            let fields = p[0].data.fields();
            vec2(to_u32(&fields[0].value), to_u32(&fields[1].value))
        },
        "`u32x3()` function" => |_| vec3(u32(0), u32(0), u32(0)),
        "`u32x3(u32, u32, u32)` function" => |p| vec3(p[0].clone(), p[1].clone(), p[2].clone()),
        "`u32x3(u32x2, u32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec3(
                to_u32(&param1_fields[0].value),
                to_u32(&param1_fields[1].value),
                p[1].clone(),
            )
        },
        "`u32x3(boolx3)` function" | "`u32x3(f32x3)` function" | "`u32x3(i32x3)` function" => |p| {
            let fields = p[0].data.fields();
            vec3(
                to_u32(&fields[0].value),
                to_u32(&fields[1].value),
                to_u32(&fields[2].value),
            )
        },
        "`u32x4()` function" => |_| vec4(u32(0), u32(0), u32(0), u32(0)),
        "`u32x4(u32, u32, u32, u32)` function" => {
            |p| vec4(p[0].clone(), p[1].clone(), p[2].clone(), p[3].clone())
        }
        "`u32x4(u32x2, u32, u32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_u32(&param1_fields[0].value),
                to_u32(&param1_fields[1].value),
                p[1].clone(),
                p[2].clone(),
            )
        },
        "`u32x4(u32x3, u32)` function" => |p| {
            let param1_fields = p[0].data.fields();
            vec4(
                to_u32(&param1_fields[0].value),
                to_u32(&param1_fields[1].value),
                to_u32(&param1_fields[2].value),
                p[1].clone(),
            )
        },
        "`u32x4(boolx4)` function" | "`u32x4(f32x4)` function" | "`u32x4(i32x4)` function" => |p| {
            let fields = p[0].data.fields();
            vec4(
                to_u32(&fields[0].value),
                to_u32(&fields[1].value),
                to_u32(&fields[2].value),
                to_u32(&fields[3].value),
            )
        },
        _ => None?,
    })
}

pub(crate) fn evaluate_fn_args<'a>(
    fn_: &dyn Node,
    args: impl Iterator<Item = &'a impl Node>,
    ctx: &mut ConstantContext<'_>,
) -> HashMap<u32, ConstantValue> {
    fn_::signature(fn_)
        .params()
        .zip(args)
        .map(|(param, arg)| {
            (
                param.id,
                arg.evaluate_constant(ctx)
                    .expect("internal error: invalid fn arg"),
            )
        })
        .collect()
}

pub(crate) fn bool(value: bool) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "u32".into(),
        data: ConstantData::Bool(value),
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
pub(crate) fn to_bool(value: &ConstantValue) -> ConstantValue {
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

pub(crate) fn f32(value: f32) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "f32".into(),
        data: ConstantData::F32(value),
    }
}

#[allow(clippy::wildcard_enum_match_arm, clippy::cast_precision_loss)]
pub(crate) fn to_f32(value: &ConstantValue) -> ConstantValue {
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

pub(crate) fn i32(value: i32) -> ConstantValue {
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
pub(crate) fn to_i32(value: &ConstantValue) -> ConstantValue {
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

pub(crate) fn u32(value: u32) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "u32".into(),
        data: ConstantData::U32(value),
    }
}

#[allow(
    clippy::wildcard_enum_match_arm,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub(crate) fn to_u32(value: &ConstantValue) -> ConstantValue {
    ConstantValue {
        transpiled_type_name: "u32".into(),
        data: match value.data {
            ConstantData::Bool(value) => ConstantData::U32(value.into()),
            ConstantData::F32(value) => ConstantData::U32(value as u32),
            ConstantData::I32(value) => ConstantData::U32(value as u32),
            ConstantData::U32(value) => ConstantData::U32(value),
            ConstantData::StructFields(_) => unreachable!("unsupported native conversion"),
        },
    }
}

pub(crate) fn vec2(x: ConstantValue, y: ConstantValue) -> ConstantData {
    ConstantData::StructFields(vec![
        struct_field("x", x.clone()),
        struct_field("y", y.clone()),
        struct_field_alias("r", x),
        struct_field_alias("g", y),
    ])
}

pub(crate) fn vec3(x: ConstantValue, y: ConstantValue, z: ConstantValue) -> ConstantData {
    ConstantData::StructFields(vec![
        struct_field("x", x.clone()),
        struct_field("y", y.clone()),
        struct_field("z", z.clone()),
        struct_field_alias("r", x),
        struct_field_alias("g", y),
        struct_field_alias("b", z),
    ])
}

pub(crate) fn vec4(
    x: ConstantValue,
    y: ConstantValue,
    z: ConstantValue,
    w: ConstantValue,
) -> ConstantData {
    ConstantData::StructFields(vec![
        struct_field("x", x.clone()),
        struct_field("y", y.clone()),
        struct_field("z", z.clone()),
        struct_field("w", w.clone()),
        struct_field_alias("r", x),
        struct_field_alias("g", y),
        struct_field_alias("b", z),
        struct_field_alias("a", w),
    ])
}

pub(crate) fn struct_field(name: &str, value: ConstantValue) -> ConstantStructFieldData {
    ConstantStructFieldData {
        name: name.into(),
        value,
        is_alias: false,
    }
}

fn struct_field_alias(name: &str, value: ConstantValue) -> ConstantStructFieldData {
    ConstantStructFieldData {
        name: name.into(),
        value,
        is_alias: true,
    }
}

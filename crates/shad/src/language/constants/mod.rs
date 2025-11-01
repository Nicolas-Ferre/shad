use crate::compilation::constant::{
    ConstantContext, ConstantData, ConstantStructFieldData, ConstantValue,
};
use crate::compilation::node::Node;
use crate::language::items::fn_;
use std::collections::HashMap;

mod native_fns_bool;
mod native_fns_f32;
mod native_fns_i32;
mod native_fns_u32;

pub(crate) fn native_fn_runner(
    fn_key: &str,
) -> Option<fn(params: &[&ConstantValue]) -> ConstantData> {
    native_fns_bool::runner(fn_key)
        .or_else(|| native_fns_f32::runner(fn_key))
        .or_else(|| native_fns_i32::runner(fn_key))
        .or_else(|| native_fns_u32::runner(fn_key))
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

fn vec2(x: ConstantValue, y: ConstantValue) -> ConstantData {
    ConstantData::StructFields(vec![
        struct_field("x", x.clone()),
        struct_field("y", y.clone()),
        struct_field_alias("r", x),
        struct_field_alias("g", y),
    ])
}

fn vec3(x: ConstantValue, y: ConstantValue, z: ConstantValue) -> ConstantData {
    ConstantData::StructFields(vec![
        struct_field("x", x.clone()),
        struct_field("y", y.clone()),
        struct_field("z", z.clone()),
        struct_field_alias("r", x),
        struct_field_alias("g", y),
        struct_field_alias("b", z),
    ])
}

fn vec4(x: ConstantValue, y: ConstantValue, z: ConstantValue, w: ConstantValue) -> ConstantData {
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

fn struct_field(name: &str, value: ConstantValue) -> ConstantStructFieldData {
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

macro_rules! const_numeric_binary_operator {
    ($fn_name:ident, $operator:tt, [$($type_:ident),+]) => {
        #[allow(clippy::integer_division)]
        fn $fn_name(left: &ConstantValue, right: &ConstantValue) -> ConstantData {
            match (&left.data, &right.data) {
                $((ConstantData::$type_(left), ConstantData::$type_(right)) => {
                    ConstantData::$type_(*left $operator *right)
                })+
                $((ConstantData::StructFields(left), ConstantData::$type_(_)) => {
                    ConstantData::StructFields(
                        left.iter()
                            .map(|left| ConstantStructFieldData {
                                name: left.name.clone(),
                                value: ConstantValue {
                                    transpiled_type_name: left.value.transpiled_type_name.clone(),
                                    data: $fn_name(&left.value, &right),
                                },
                                is_alias: left.is_alias,
                            })
                            .collect(),
                    )
                })+
                $((ConstantData::$type_(_), ConstantData::StructFields(right)) => {
                    ConstantData::StructFields(
                        right.iter()
                            .map(|right| ConstantStructFieldData {
                                name: right.name.clone(),
                                value: ConstantValue {
                                    transpiled_type_name: right.value.transpiled_type_name.clone(),
                                    data: $fn_name(&left, &right.value),
                                },
                                is_alias: right.is_alias,
                            })
                            .collect(),
                    )
                })+
                (ConstantData::StructFields(left), ConstantData::StructFields(right)) => {
                    ConstantData::StructFields(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| ConstantStructFieldData {
                                name: left.name.clone(),
                                value: ConstantValue {
                                    transpiled_type_name: left.value.transpiled_type_name.clone(),
                                    data: $fn_name(&left.value, &right.value),
                                },
                                is_alias: left.is_alias,
                            })
                            .collect(),
                    )
                }
                (_, _) => unreachable!("invalid const operands"),
            }
        }
    };
}

macro_rules! const_bool_binary_operator {
    ($fn_name:ident, $operator:tt, [$($type_:ident),+]) => {
        fn $fn_name(left: &ConstantValue, right: &ConstantValue) -> ConstantData {
            match (&left.data, &right.data) {
                $((ConstantData::$type_(left), ConstantData::$type_(right)) => {
                    ConstantData::Bool(*left $operator *right)
                })+
                (ConstantData::StructFields(left), ConstantData::StructFields(right)) => {
                    ConstantData::StructFields(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| ConstantStructFieldData {
                                name: left.name.clone(),
                                value: ConstantValue {
                                    transpiled_type_name: "u32".into(),
                                    data: $fn_name(&left.value, &right.value),
                                },
                                is_alias: left.is_alias,
                            })
                            .collect(),
                    )
                }
                (_, _) => unreachable!("invalid const operands"),
            }
        }
    };
}

macro_rules! const_unary_operator {
    ($fn_name:ident, $operator:tt, [$($type_:ident),+]) => {
        fn $fn_name(value: &ConstantValue) -> ConstantData {
            match &value.data {
                $(ConstantData::$type_(value) => ConstantData::$type_($operator *value),)+
                ConstantData::StructFields(fields) => {
                    ConstantData::StructFields(
                        fields.iter()
                            .map(|field| ConstantStructFieldData {
                                name: field.name.clone(),
                                value: ConstantValue {
                                    transpiled_type_name: field.value.transpiled_type_name.clone(),
                                    data: $fn_name(&field.value),
                                },
                                is_alias: field.is_alias,
                            })
                            .collect(),
                    )
                }
                _ => unreachable!("invalid const operands"),
            }
        }
    };
}

const_numeric_binary_operator!(add, +, [F32, I32, U32]);
const_numeric_binary_operator!(sub, -, [F32, I32, U32]);
const_numeric_binary_operator!(mul, *, [F32, I32, U32]);
const_numeric_binary_operator!(div, /, [F32, I32, U32]);
const_numeric_binary_operator!(mod_, %, [I32, U32]);
const_bool_binary_operator!(lt, <, [Bool, F32, I32, U32]);
const_bool_binary_operator!(gt, >, [Bool, F32, I32, U32]);
const_bool_binary_operator!(le, <=, [Bool, F32, I32, U32]);
const_bool_binary_operator!(ge, >=, [Bool, F32, I32, U32]);
const_bool_binary_operator!(eq, ==, [Bool, F32, I32, U32]);
const_bool_binary_operator!(ne, !=, [Bool, F32, I32, U32]);
const_bool_binary_operator!(and, &&, [Bool]);
const_bool_binary_operator!(or, ||, [Bool]);
const_unary_operator!(neg, -, [F32, I32]);
const_unary_operator!(not, !, [Bool]);

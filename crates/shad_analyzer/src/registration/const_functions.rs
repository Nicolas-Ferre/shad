use crate::registration::constants::ConstantValue;
use crate::{Analysis, TypeId};

/// The implementation of a `const` function.
pub type ConstFn = fn(&[ConstantValue]) -> ConstantValue;

/// The unique identifier of a `const` function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstFnId {
    /// The function name.
    pub name: String,
    /// The function parameter types.
    pub param_types: Vec<ConstFnParamType>,
}

/// The type of `const` function parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstFnParamType {
    /// The `u32` type.
    U32,
    /// The `i32` type.
    I32,
    /// The `f32` type.
    F32,
    /// The `bool` type.
    Bool,
}

impl ConstFnParamType {
    pub(crate) fn from_type_id(type_id: &TypeId) -> Option<Self> {
        match type_id.as_str() {
            "u32" => Some(Self::U32),
            "i32" => Some(Self::I32),
            "f32" => Some(Self::F32),
            "bool" => Some(Self::Bool),
            _ => None,
        }
    }

    // coverage: off (simple and not critical logic)
    pub(crate) fn zero_value(self) -> ConstantValue {
        match self {
            Self::U32 => ConstantValue::U32(0),
            Self::I32 => ConstantValue::I32(0),
            Self::F32 => ConstantValue::F32(0.),
            Self::Bool => ConstantValue::Bool(false),
        }
    }
    // coverage: on
}

macro_rules! unary_operator {
    ($fn_name:literal, $type_variant:ident, $operator:tt) => {
        (
            ConstFnId {
                name: $fn_name.into(),
                param_types: vec![ConstFnParamType::$type_variant],
            },
            (|values| {
                if let ConstantValue::$type_variant(value) = &values[0] {
                    ConstantValue::$type_variant($operator value)
                } else {
                    unreachable!("internal error: invalid const function param types")
                }
            }) as ConstFn,
        )
    };
}

macro_rules! binary_operator {
    ($fn_name:literal, $type_variant:ident, $operator:tt) => {
        (
            ConstFnId {
                name: $fn_name.into(),
                param_types: vec![ConstFnParamType::$type_variant, ConstFnParamType::$type_variant],
            },
            (|values| {
                if let (ConstantValue::$type_variant(left), ConstantValue::$type_variant(right)) =
                    (&values[0], &values[1])
                {
                    ConstantValue::$type_variant(left $operator right)
                } else {
                    unreachable!("internal error: invalid const function param types")
                }
            }) as ConstFn,
        )
    };
}

pub(crate) fn register(analysis: &mut Analysis) {
    analysis.const_functions = [
        binary_operator!("__add__", U32, +),
        binary_operator!("__sub__", U32, -),
        binary_operator!("__mul__", U32, *),
        binary_operator!("__div__", U32, /),
        binary_operator!("__mod__", U32, %),
        binary_operator!("__add__", I32, +),
        binary_operator!("__sub__", I32, -),
        binary_operator!("__mul__", I32, *),
        binary_operator!("__div__", I32, /),
        binary_operator!("__mod__", I32, %),
        binary_operator!("__add__", F32, +),
        binary_operator!("__sub__", F32, -),
        binary_operator!("__mul__", F32, *),
        binary_operator!("__div__", F32, /),
        unary_operator!("__neg__", I32, -),
        unary_operator!("__neg__", F32, -),
    ]
    .into_iter()
    .collect();
}

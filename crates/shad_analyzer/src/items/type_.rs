use fxhash::FxHashMap;
use std::rc::Rc;

/// The 32-bit floating point type name.
pub const F32_TYPE: &str = "f32";
/// The 32-bit unsigned integer type name.
pub const U32_TYPE: &str = "u32";
/// The 32-bit signed integer type name.
pub const I32_TYPE: &str = "i32";
/// The boolean type name.
pub const BOOL_TYPE: &str = "bool";

/// An analyzed type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgType {
    /// The type name in the initial Shad code.
    pub name: AsgTypeName,
    /// The final name that will be used for buffers.
    pub buf_final_name: String,
    /// The final name that will be used for variables.
    pub expr_final_name: String,
    /// The size in bytes of the type.
    pub size: usize,
}

/// The name of an analyzed type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgTypeName {
    /// A primitive type.
    Primitive(&'static str),
}

impl AsgTypeName {
    /// Returns the type name as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Primitive(name) => name,
        }
    }
}

pub(crate) fn primitive_types() -> FxHashMap<String, Rc<AsgType>> {
    [
        (
            F32_TYPE.into(),
            Rc::new(AsgType {
                name: AsgTypeName::Primitive(F32_TYPE),
                buf_final_name: F32_TYPE.into(),
                expr_final_name: F32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            U32_TYPE.into(),
            Rc::new(AsgType {
                name: AsgTypeName::Primitive(U32_TYPE),
                buf_final_name: U32_TYPE.into(),
                expr_final_name: U32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            I32_TYPE.into(),
            Rc::new(AsgType {
                name: AsgTypeName::Primitive(I32_TYPE),
                buf_final_name: I32_TYPE.into(),
                expr_final_name: I32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            BOOL_TYPE.into(),
            Rc::new(AsgType {
                name: AsgTypeName::Primitive(BOOL_TYPE),
                buf_final_name: U32_TYPE.into(),
                expr_final_name: BOOL_TYPE.into(),
                size: 4,
            }),
        ),
    ]
    .into_iter()
    .collect()
}

use crate::Asg;
use fxhash::FxHashMap;
use std::rc::Rc;

const UNDEFINED_TYPE: &str = "<undefined>";
const F32_TYPE: &str = "f32";
const U32_TYPE: &str = "u32";
const I32_TYPE: &str = "i32";

pub(crate) fn primitive_types() -> FxHashMap<String, Rc<AsgType>> {
    [
        (
            UNDEFINED_TYPE.into(),
            Rc::new(AsgType {
                final_name: UNDEFINED_TYPE.into(),
                size: 0,
            }),
        ),
        (
            F32_TYPE.into(),
            Rc::new(AsgType {
                final_name: F32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            U32_TYPE.into(),
            Rc::new(AsgType {
                final_name: U32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            I32_TYPE.into(),
            Rc::new(AsgType {
                final_name: I32_TYPE.into(),
                size: 4,
            }),
        ),
    ]
    .into_iter()
    .collect()
}

pub(crate) fn undefined(asg: &Asg) -> &Rc<AsgType> {
    &asg.types[UNDEFINED_TYPE]
}

pub(crate) fn find<'a>(asg: &'a Asg, type_name: &str) -> &'a Rc<AsgType> {
    asg.types
        .get(type_name)
        .unwrap_or_else(|| &asg.types[UNDEFINED_TYPE])
}

/// An analyzed type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgType {
    /// The final name that will be used in shaders.
    pub final_name: String,
    /// The size in bytes of the type.
    pub size: usize,
}

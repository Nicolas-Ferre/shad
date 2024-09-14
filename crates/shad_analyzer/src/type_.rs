use crate::Asg;
use fxhash::FxHashMap;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::AstIdent;
use std::rc::Rc;

const UNDEFINED_TYPE: &str = "<undefined>";
const F32_TYPE: &str = "f32";
const U32_TYPE: &str = "u32";
const I32_TYPE: &str = "i32";

/// An analyzed type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgType {
    /// The type name in the initial Shad code.
    pub name: Option<AstIdent>,
    /// The final name that will be used in shaders.
    pub final_name: String,
    /// The size in bytes of the type.
    pub size: usize,
}

impl AsgType {
    pub(crate) fn name(&self) -> &str {
        self.name
            .as_ref()
            .map_or_else(|| &self.final_name, |name| &name.label)
    }
}

pub(crate) fn primitive_types() -> FxHashMap<String, Rc<AsgType>> {
    [
        (
            UNDEFINED_TYPE.into(),
            Rc::new(AsgType {
                name: None,
                final_name: UNDEFINED_TYPE.into(),
                size: 0,
            }),
        ),
        (
            F32_TYPE.into(),
            Rc::new(AsgType {
                name: None,
                final_name: F32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            U32_TYPE.into(),
            Rc::new(AsgType {
                name: None,
                final_name: U32_TYPE.into(),
                size: 4,
            }),
        ),
        (
            I32_TYPE.into(),
            Rc::new(AsgType {
                name: None,
                final_name: I32_TYPE.into(),
                size: 4,
            }),
        ),
    ]
    .into_iter()
    .collect()
}

// coverage: off (unreachable in `shad_runner` crate)
pub(crate) fn undefined(asg: &Asg) -> &Rc<AsgType> {
    &asg.types[UNDEFINED_TYPE]
}
// coverage: on

pub(crate) fn find<'a>(asg: &'a mut Asg, name: &AstIdent) -> &'a Rc<AsgType> {
    if let Some(type_) = asg.types.get(&name.label) {
        type_
    } else {
        asg.errors.push(not_found_type_error(asg, name));
        &asg.types[UNDEFINED_TYPE]
    }
}

fn not_found_type_error(asg: &Asg, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` type", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined type".into(),
        }],
        &asg.code,
        &asg.path,
    )
}

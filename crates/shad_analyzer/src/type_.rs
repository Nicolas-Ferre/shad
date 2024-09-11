use crate::AnalyzedBuffers;
use fxhash::FxHashMap;
use shad_parser::AstLiteralType;
use std::rc::Rc;

const UNDEFINED_TYPE: &str = "<undefined>";
const F32_TYPE: &str = "f32";
const U32_TYPE: &str = "u32";
const I32_TYPE: &str = "i32";

/// All types found when analysing a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedTypes {
    /// The types.
    pub types: FxHashMap<String, Rc<Type>>,
}

impl AnalyzedTypes {
    pub(crate) fn new() -> Self {
        Self {
            types: [
                (
                    UNDEFINED_TYPE.into(),
                    Rc::new(Type {
                        final_name: UNDEFINED_TYPE.into(),
                        size: 0,
                    }),
                ),
                (
                    F32_TYPE.into(),
                    Rc::new(Type {
                        final_name: F32_TYPE.into(),
                        size: 4,
                    }),
                ),
                (
                    U32_TYPE.into(),
                    Rc::new(Type {
                        final_name: U32_TYPE.into(),
                        size: 4,
                    }),
                ),
                (
                    I32_TYPE.into(),
                    Rc::new(Type {
                        final_name: I32_TYPE.into(),
                        size: 4,
                    }),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub(crate) fn expr_type(
        &self,
        expr: &shad_parser::AstExpr,
        buffers: &AnalyzedBuffers,
    ) -> Rc<Type> {
        match expr {
            shad_parser::AstExpr::Literal(literal) => match literal.type_ {
                AstLiteralType::F32 => self.types[F32_TYPE].clone(),
                AstLiteralType::U32 => self.types[U32_TYPE].clone(),
                AstLiteralType::I32 => self.types[I32_TYPE].clone(),
            },
            shad_parser::AstExpr::Ident(ident) => match buffers.find(&ident.label) {
                Some(buffer) => buffer.type_.clone(),
                None => self.types[UNDEFINED_TYPE].clone(),
            },
        }
    }
}

/// An analyzed type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    /// The final name that will be used in shaders.
    pub final_name: String,
    /// The size in bytes of the type.
    pub size: usize,
}

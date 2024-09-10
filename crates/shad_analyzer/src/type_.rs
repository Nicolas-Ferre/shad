use fxhash::FxHashMap;
use shad_parser::LiteralType;
use std::rc::Rc;

const F32_TYPE: &str = "f32";
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
                    F32_TYPE.into(),
                    Rc::new(Type {
                        final_name: F32_TYPE.into(),
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

    pub(crate) fn expr_type(&self, expr: &shad_parser::Expr) -> &Rc<Type> {
        match expr {
            shad_parser::Expr::Literal(literal) => match literal.type_ {
                LiteralType::F32 => &self.types[F32_TYPE],
                LiteralType::I32 => &self.types[I32_TYPE],
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

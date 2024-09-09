use fxhash::FxHashMap;
use shad_parser::LiteralType;
use std::iter;
use std::rc::Rc;

const FLOAT_TYPE: &str = "float";

/// All types found when analysing a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedTypes {
    /// The types.
    pub types: FxHashMap<String, Rc<Type>>,
}

impl AnalyzedTypes {
    pub(crate) fn new() -> Self {
        Self {
            types: iter::once((
                FLOAT_TYPE.into(),
                Rc::new(Type {
                    final_name: "f32".into(),
                    size: 4,
                }),
            ))
            .collect(),
        }
    }

    pub(crate) fn expr_type(&self, expr: &shad_parser::Expr) -> &Rc<Type> {
        match expr {
            shad_parser::Expr::Literal(literal) => match literal.type_ {
                LiteralType::Float => &self.types[FLOAT_TYPE],
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

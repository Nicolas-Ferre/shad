use crate::{AnalyzedBuffers, Buffer, ErrorLevel, LocatedMessage, SemanticError, Type};
use shad_parser::{LiteralType, ParsedProgram, Span};
use std::rc::Rc;
use std::str::FromStr;

/// All compute shaders run at startup generated from a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GeneratedInitComputeShaders {
    /// The generated shaders.
    pub shaders: Vec<ComputeShader>,
    /// The semantic errors related to statements.
    pub errors: Vec<SemanticError>,
}

impl GeneratedInitComputeShaders {
    const F32_INT_PART_LIMIT: usize = 38;

    pub(crate) fn new(parsed: &ParsedProgram, buffers: &AnalyzedBuffers) -> Self {
        let mut shaders = vec![];
        let mut errors = vec![];
        for buffer in &buffers.buffers {
            shaders.push(ComputeShader {
                buffers: vec![buffer.clone()],
                statements: vec![match &buffer.value {
                    shad_parser::Expr::Literal(literal) => Statement::Assignment(Assignment {
                        assigned: Value::Buffer(buffer.clone()),
                        value: {
                            let final_value = literal.value.replace('_', "");
                            errors.extend(Self::literal_error(literal, &final_value, parsed));
                            Expr::Literal(Literal {
                                value: final_value,
                                type_: buffer.type_.clone(),
                            })
                        },
                    }),
                }],
                name: format!("buffer_init:{}", buffer.name.label),
            });
        }
        Self { shaders, errors }
    }

    fn literal_error(
        literal: &shad_parser::Literal,
        final_value: &str,
        parsed: &ParsedProgram,
    ) -> Option<SemanticError> {
        match literal.type_ {
            LiteralType::F32 => Self::f32_literal_error(literal, final_value, parsed),
            LiteralType::U32 => {
                let digits = &final_value[..final_value.len() - 1];
                Self::int_literal_error::<u32>(literal, digits, "u32", parsed)
            }
            LiteralType::I32 => Self::int_literal_error::<i32>(literal, final_value, "i32", parsed),
        }
    }

    fn f32_literal_error(
        literal: &shad_parser::Literal,
        final_value: &str,
        parsed: &ParsedProgram,
    ) -> Option<SemanticError> {
        let digit_count = final_value
            .find('.')
            .expect("internal error: `.` not found in `f32` literal");
        (digit_count > Self::F32_INT_PART_LIMIT).then(|| {
            let span = Span::new(literal.span.start, literal.span.start + digit_count);
            SemanticError::new(
                "`f32` literal with too many digits in integer part",
                vec![
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span,
                        text: format!("found {digit_count} digits"),
                    },
                    LocatedMessage {
                        level: ErrorLevel::Info,
                        span,
                        text: format!("maximum {} digits are expected", Self::F32_INT_PART_LIMIT),
                    },
                ],
                parsed,
            )
        })
    }

    fn int_literal_error<T>(
        literal: &shad_parser::Literal,
        final_value: &str,
        type_name: &str,
        parsed: &ParsedProgram,
    ) -> Option<SemanticError>
    where
        T: FromStr,
    {
        let is_literal_invalid = T::from_str(final_value).is_err();
        is_literal_invalid.then(|| {
            SemanticError::new(
                format!("`{type_name}` literal out of range"),
                vec![LocatedMessage {
                    level: ErrorLevel::Error,
                    span: literal.span,
                    text: format!("value is outside allowed range for `{type_name}` type"),
                }],
                parsed,
            )
        })
    }
}

/// A compute shader definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ComputeShader {
    /// The buffers to bind to the shader.
    pub buffers: Vec<Rc<Buffer>>,
    /// The statements of the shader main function.
    pub statements: Vec<Statement>,
    /// The name of the shader.
    pub name: String,
}

/// A statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    /// A variable assignment.
    Assignment(Assignment),
}

/// An assignment statement definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
    /// A statement definition.
    pub assigned: Value,
    /// The assigned value.
    pub value: Expr,
}

/// A value definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    /// A buffer.
    Buffer(Rc<Buffer>),
}

/// An expression definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    /// A literal.
    Literal(Literal),
}

/// A literal value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Literal {
    /// The literal value.
    pub value: String,
    /// The literal type.
    pub type_: Rc<Type>,
}

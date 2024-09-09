use crate::{AnalyzedBuffers, Buffer, ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{Literal, ParsedProgram, Span};
use std::rc::Rc;

/// All compute shaders run at startup generated from a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GeneratedInitComputeShaders {
    /// The generated shaders.
    pub shaders: Vec<ComputeShader>,
    /// The semantic errors related to statements.
    pub errors: Vec<SemanticError>,
}

impl GeneratedInitComputeShaders {
    const FLOAT_INT_PART_LIMIT: usize = 38;

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
                            errors.extend(Self::too_many_float_digits_error(literal, parsed));
                            Expr::Literal(literal.value.replace('_', ""))
                        },
                    }),
                }],
                name: format!("buffer_init:{}", buffer.name.label),
            });
        }
        Self { shaders, errors }
    }

    fn too_many_float_digits_error(
        literal: &Literal,
        parsed: &ParsedProgram,
    ) -> Option<SemanticError> {
        let digit_count = literal
            .value
            .find('.')
            .expect("internal error: `.` not found in float literal");
        (digit_count > Self::FLOAT_INT_PART_LIMIT).then(|| {
            let span = Span::new(literal.span.start, literal.span.start + digit_count);
            SemanticError::new(
                "float literal with too many digits in integer part",
                vec![
                    LocatedMessage {
                        level: ErrorLevel::Error,
                        span,
                        text: format!("found {digit_count} digits"),
                    },
                    LocatedMessage {
                        level: ErrorLevel::Info,
                        span,
                        text: format!("maximum {} digits are expected", Self::FLOAT_INT_PART_LIMIT),
                    },
                ],
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
    Literal(String),
}

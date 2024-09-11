use crate::{AnalyzedBuffers, Buffer, ErrorLevel, LocatedMessage, SemanticError, Type};
use shad_parser::{Ast, AstLiteralType, Span};
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

    pub(crate) fn new(ast: &Ast, buffers: &AnalyzedBuffers) -> Self {
        let mut shaders = vec![];
        let mut errors = vec![];
        for buffer in &buffers.buffers {
            shaders.push(ComputeShader {
                buffers: Self::buffers(buffer, buffers),
                statements: Self::statements(buffer, buffers, ast, &mut errors),
                name: format!("buffer_init:{}", buffer.name.label),
            });
        }
        Self { shaders, errors }
    }

    fn buffers(current_buffer: &Rc<Buffer>, buffers: &AnalyzedBuffers) -> Vec<Rc<Buffer>> {
        match &current_buffer.value {
            shad_parser::AstExpr::Literal(_) => vec![current_buffer.clone()],
            shad_parser::AstExpr::Ident(ident) => match buffers.find(&ident.label) {
                Some(assigned_buffer) => vec![current_buffer.clone(), assigned_buffer.clone()],
                None => vec![],
            },
        }
    }

    fn statements(
        current_buffer: &Rc<Buffer>,
        buffers: &AnalyzedBuffers,
        ast: &Ast,
        errors: &mut Vec<SemanticError>,
    ) -> Vec<Statement> {
        match &current_buffer.value {
            shad_parser::AstExpr::Literal(literal) => {
                vec![Statement::Assignment(Assignment {
                    assigned: Value::Buffer(current_buffer.clone()),
                    value: {
                        let final_value = literal.value.replace('_', "");
                        errors.extend(Self::literal_error(literal, &final_value, ast));
                        Expr::Literal(Literal {
                            value: final_value,
                            type_: current_buffer.type_.clone(),
                        })
                    },
                })]
            }
            shad_parser::AstExpr::Ident(ident) => {
                if let Some(assigned_buffer) = buffers.find(&ident.label) {
                    if current_buffer.index < assigned_buffer.index {
                        errors.push(Self::not_found_ident(ident, ast));
                    }
                    vec![Statement::Assignment(Assignment {
                        assigned: Value::Buffer(current_buffer.clone()),
                        value: { Expr::Ident(Ident::Buffer(assigned_buffer.clone())) },
                    })]
                } else {
                    errors.push(Self::not_found_ident(ident, ast));
                    vec![]
                }
            }
        }
    }

    fn literal_error(
        literal: &shad_parser::AstLiteral,
        final_value: &str,
        ast: &Ast,
    ) -> Option<SemanticError> {
        match literal.type_ {
            AstLiteralType::F32 => Self::f32_literal_error(literal, final_value, ast),
            AstLiteralType::U32 => {
                let digits = &final_value[..final_value.len() - 1];
                Self::int_literal_error::<u32>(literal, digits, "u32", ast)
            }
            AstLiteralType::I32 => Self::int_literal_error::<i32>(literal, final_value, "i32", ast),
        }
    }

    fn not_found_ident(ident: &shad_parser::AstIdent, ast: &Ast) -> SemanticError {
        SemanticError::new(
            format!("could not find `{}` value", ident.label),
            vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: ident.span,
                text: "undefined identifier".into(),
            }],
            ast,
        )
    }

    fn f32_literal_error(
        literal: &shad_parser::AstLiteral,
        final_value: &str,
        ast: &Ast,
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
                ast,
            )
        })
    }

    fn int_literal_error<T>(
        literal: &shad_parser::AstLiteral,
        final_value: &str,
        type_name: &str,
        ast: &Ast,
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
                ast,
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
    /// An identifier.
    Ident(Ident),
}

/// A literal value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Literal {
    /// The literal value.
    pub value: String,
    /// The literal type.
    pub type_: Rc<Type>,
}

/// An identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ident {
    /// A buffer identifier.
    Buffer(Rc<Buffer>),
}

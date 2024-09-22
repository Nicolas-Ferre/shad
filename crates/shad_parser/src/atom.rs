use crate::token::{Token, TokenType};
use logos::Lexer;
use shad_error::{Span, SyntaxError};

/// A parsed identifier.
///
/// It generally corresponds to a variable or function name.
///
/// An identifier matches the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
///
/// # Examples
///
/// - `my_buffer` will be parsed as an identifier in `buf my_buffer = 0;` Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstIdent {
    /// The span of the identifier.
    pub span: Span,
    /// The identifier as a string.
    pub label: String,
}

impl AstIdent {
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = parse_token(lexer, TokenType::Ident)?;
        Ok(Self {
            span: token.span,
            label: token.slice.to_string(),
        })
    }
}

/// A parsed literal.
///
/// The following literals are recognized:
/// - `f32` literal, following regex `[0-9][0-9_]*\\.([0-9][0-9_]*)?`.
/// - `u32` literal, following regex `[0-9][0-9_]*u`.
/// - `i32` literal, following regex `[0-9][0-9_]*`.
///
/// # Examples
///
/// - Shad code `1.` will be parsed as an `f32` literal.
/// - Shad code `1.2` will be parsed as an `f32` literal.
/// - Shad code `1_000.420_456` will be parsed as an `f32` literal.
/// - Shad code `123u` will be parsed as a `u32` literal.
/// - Shad code `123` will be parsed as an `i32` literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstLiteral {
    /// The span of the literal.
    pub span: Span,
    /// The value of the literal.
    pub value: String,
    /// The type of the literal.
    pub type_: AstLiteralType,
}

impl AstLiteral {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(
        lexer: &mut Lexer<'_, TokenType>,
        type_: TokenType,
    ) -> Result<Self, SyntaxError> {
        let token = parse_token(lexer, type_)?;
        Ok(Self {
            span: token.span,
            value: token.slice.to_string(),
            type_: match type_ {
                TokenType::F32Literal => AstLiteralType::F32,
                TokenType::U32Literal => AstLiteralType::U32,
                TokenType::I32Literal => AstLiteralType::I32,
                TokenType::True | TokenType::False => AstLiteralType::Bool,
                _ => unreachable!("internal error: not supported literal"),
            },
        })
    }
}

/// The type of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstLiteralType {
    /// The `f32` primitive type.
    F32,
    /// The `u32` primitive type.
    U32,
    /// The `i32` primitive type.
    I32,
    /// The `bool` primitive type.
    Bool,
}

pub(crate) fn parse_token<'a>(
    lexer: &mut Lexer<'a, TokenType>,
    expected_type: TokenType,
) -> Result<Token<'a>, SyntaxError> {
    let token = Token::next(lexer)?;
    if token.type_ == expected_type {
        Ok(token)
    } else {
        Err(SyntaxError::new(
            token.span.start,
            format!("expected {}", expected_type.label()),
        ))
    }
}

use crate::common::{Span, Token, TokenType};
use crate::SyntaxError;
use logos::Lexer;

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
pub struct Ident {
    /// The span of the identifier.
    pub span: Span,
    /// The identifier as a string.
    pub label: String,
}

impl Ident {
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = parse_token(lexer, TokenType::Ident)?;
        Ok(Self {
            span: Span::from_logos(token.span),
            label: token.slice.to_string(),
        })
    }
}

/// A parsed literal.
///
/// The following literals are recognized:
/// - `float` literal, following regex `[0-9][0-9_]*\\.([0-9][0-9_]*)?`.
///
/// # Examples
///
/// - Shad code `1.` will be parsed as a `float` literal.
/// - Shad code `1.2` will be parsed as a `float` literal.
/// - Shad code `1_000.420_456` will be parsed as a `float` literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The span of the literal.
    pub span: Span,
    /// The cleaned value of the literal.
    ///
    /// All leading zeros are removed from an integer literal (except for `0` literal).
    pub value: String,
    /// The type of the literal.
    pub type_: LiteralType,
}

impl Literal {
    pub(crate) fn parse_float(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = parse_token(lexer, TokenType::FloatLiteral)?;
        Ok(Self {
            span: Span::from_logos(token.span),
            value: token.slice.to_string(),
            type_: LiteralType::Float,
        })
    }
}

/// The type of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralType {
    /// The `float` primitive type.
    Float,
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

use crate::common::{Span, Token, TokenType};
use crate::ParsingError;
use logos::Lexer;

/// A parsed identifier.
///
/// It generally corresponds to a variable or function name.
///
/// An identifier matches the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
///
/// # Examples
///
/// `my_func` will be parsed as an identifier in the following Shad code:
/// ```custom,rust
/// fn my_func() {}
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    /// The span.
    pub span: Span,
    /// The identifier as a string.
    pub label: String,
}

impl Ident {
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let token = parse_token(lexer, TokenType::Ident)?;
        Ok(Self {
            span: Span::new(token.span),
            label: token.slice.to_string(),
        })
    }
}

/// A parsed literal.
///
/// The following literals are recognized:
/// - `float` literal, following regex `[0-9][0-9_]*\\.([0-9][0-9_]*)?`.
/// - `int` literal, following regex `[a-zA-Z_][a-zA-Z0-9_]*` and representing a decimal value.
///
/// # Examples
///
/// `1_000_000` will be parsed as an integer literal in the following Shad code:
/// ```custom,rust
/// let value = 1_000_000;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The span.
    pub span: Span,
    /// The cleaned value of the literal.
    ///
    /// All leading zeros are removed from an integer literal (except for `0` literal).
    pub value: String,
    /// The type of the literal.
    pub type_: LiteralType,
}

impl Literal {
    pub(crate) fn parse_float(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let token = parse_token(lexer, TokenType::FloatLiteral)?;
        Ok(Self {
            span: Span::new(token.span),
            value: token.slice.to_string(),
            type_: LiteralType::Float,
        })
    }

    pub(crate) fn parse_int(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let token = parse_token(lexer, TokenType::IntLiteral)?;
        let mut chars: Vec<_> = token.slice.chars().collect();
        while chars.first() == Some(&'0') && chars.get(1).is_some() {
            chars.remove(0);
        }
        Ok(Self {
            span: Span::new(token.span),
            value: chars.iter().collect(),
            type_: LiteralType::Int,
        })
    }
}

/// The type of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralType {
    /// `float` primitive type.
    Float,
    /// `int` primitive type.
    Int,
}

pub(crate) fn parse_token<'a>(
    lexer: &mut Lexer<'a, TokenType>,
    expected_type: TokenType,
) -> Result<Token<'a>, ParsingError> {
    let token = Token::next(lexer)?;
    if token.type_ == expected_type {
        Ok(token)
    } else {
        Err(ParsingError::new(
            token.span.start,
            format!("expected {}", expected_type.label()),
        ))
    }
}

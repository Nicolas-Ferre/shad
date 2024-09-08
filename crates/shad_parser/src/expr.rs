use crate::common::{Token, TokenType};
use crate::{Literal, Span};
use logos::Lexer;
use crate::error::SyntaxError;

/// A parsed expression.
///
/// # Examples
///
/// - Shad code `3.14` will be parsed as an [`Expr::Literal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A literal.
    Literal(Literal),
}

impl Expr {
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(expr) => expr.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::FloatLiteral => Ok(Self::Literal(Literal::parse_float(lexer)?)),
            _ => Err(SyntaxError::new(token.span.start, "expected expression")),
        }
    }
}
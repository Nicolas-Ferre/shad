use crate::common::{Token, TokenType};
use crate::error::SyntaxError;
use crate::Literal;
use logos::Lexer;

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
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            type_ @ (TokenType::F32Literal | TokenType::I32Literal) => {
                Ok(Self::Literal(Literal::parse(lexer, type_)?))
            }
            _ => Err(SyntaxError::new(token.span.start, "expected expression")),
        }
    }
}

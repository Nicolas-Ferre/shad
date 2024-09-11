use crate::common::{Token, TokenType};
use crate::error::SyntaxError;
use crate::{AstIdent, AstLiteral};
use logos::Lexer;

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpr {
    /// A literal.
    Literal(AstLiteral),
    /// An identifier.
    Ident(AstIdent),
}

impl AstExpr {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            type_ @ (TokenType::F32Literal | TokenType::U32Literal | TokenType::I32Literal) => {
                Ok(Self::Literal(AstLiteral::parse(lexer, type_)?))
            }
            TokenType::Ident => Ok(Self::Ident(AstIdent::parse(lexer)?)),
            _ => Err(SyntaxError::new(token.span.start, "expected expression")),
        }
    }
}

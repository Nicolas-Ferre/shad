use crate::token::{Token, TokenType};
use crate::{AstFnCall, AstIdent};
use logos::Lexer;
use shad_error::{Span, SyntaxError};

/// A parsed left value that can be assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstLeftValue {
    /// An identifier.
    Ident(AstIdent),
    /// A function call.
    FnCall(AstFnCall),
}

impl AstLeftValue {
    /// Returns the span of the value.
    pub fn span(&self) -> Span {
        match self {
            Self::Ident(value) => value.span,
            Self::FnCall(value) => value.span,
        }
    }

    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let tmp_lexer = &mut lexer.clone();
        let token = Token::next(tmp_lexer)?;
        let next_token = Token::next(tmp_lexer)?;
        match token.type_ {
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(AstFnCall::parse(lexer)?))
                } else {
                    Ok(Self::Ident(AstIdent::parse(lexer)?))
                }
            }
            _ => Err(SyntaxError::new(token.span.start, "expected left value")),
        }
    }
}

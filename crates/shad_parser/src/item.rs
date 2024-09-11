use crate::atom::parse_token;
use crate::common::{Token, TokenType};
use crate::error::SyntaxError;
use crate::{AstExpr, AstIdent, Span};
use logos::Lexer;

/// A parsed item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstItem {
    /// A buffer definition.
    Buffer(AstBufferItem),
}

impl AstItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Buf => Ok(Self::Buffer(AstBufferItem::parse(lexer)?)),
            _ => Err(SyntaxError::new(token.span.start, "expected item")),
        }
    }
}

/// A parsed buffer definition.
///
/// # Examples
///
/// Shad code `buf my_buffer = 2;` will be parsed as a buffer definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstBufferItem {
    /// The span of the buffer definition.
    pub span: Span,
    /// The name of the buffer.
    pub name: AstIdent,
    /// The initial value of the buffer.
    pub value: AstExpr,
}

impl AstBufferItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let buf_ = parse_token(lexer, TokenType::Buf)?;
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Equal)?;
        let value = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span {
                start: buf_.span.start,
                end: semi_colon.span.end,
            },
            name,
            value,
        })
    }
}

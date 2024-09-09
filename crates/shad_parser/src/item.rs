use crate::atom::parse_token;
use crate::common::{Token, TokenType};
use crate::error::SyntaxError;
use crate::{Expr, Ident, Span};
use logos::Lexer;

/// A parsed item.
///
/// # Examples
///
/// - Shad code `buf my_buffer = 2;` will be parsed as an [`Item::Buffer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A buffer definition.
    Buffer(BufferItem),
}

impl Item {
    /// Returns the span of the item.
    pub fn span(&self) -> Span {
        match self {
            Self::Buffer(item) => item.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Buf => Ok(Self::Buffer(BufferItem::parse(lexer)?)),
            _ => Err(SyntaxError::new(token.span.start, "expected item")),
        }
    }
}

/// A parsed buffer definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferItem {
    /// The span of the buffer definition.
    pub span: Span,
    /// The name of the buffer.
    pub name: Ident,
    /// The initial value of the buffer.
    pub value: Expr,
}

impl BufferItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let buf_ = parse_token(lexer, TokenType::Buf)?;
        let name = Ident::parse(lexer)?;
        parse_token(lexer, TokenType::Equal)?;
        let value = Expr::parse(lexer)?;
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

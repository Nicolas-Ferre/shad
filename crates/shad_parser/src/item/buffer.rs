use crate::atom::parse_token;
use crate::token::{Lexer, TokenType};
use crate::{AstExpr, AstIdent, AstIdentType};
use shad_error::SyntaxError;

/// A parsed buffer definition.
///
/// # Examples
///
/// Shad code `buf my_buffer = 2;` will be parsed as a buffer definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstBufferItem {
    /// The name of the buffer.
    pub name: AstIdent,
    /// The initial value of the buffer.
    pub value: AstExpr,
    /// Whether the item is public.
    pub is_pub: bool,
}

impl AstBufferItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Buf)?;
        let name = AstIdent::parse(lexer, AstIdentType::BufDef)?;
        parse_token(lexer, TokenType::Assigment)?;
        let value = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            name,
            value,
            is_pub,
        })
    }
}
use crate::atom::parse_token;
use crate::token::{Lexer, TokenType};
use crate::{AstExpr, AstIdent};
use shad_error::SyntaxError;

/// A parsed constant definition.
///
/// # Examples
///
/// Shad code `const MY_CONSTANT = 2;` will be parsed as a constant definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstConstItem {
    /// The name of the constant.
    pub name: AstIdent,
    /// The initial value of the constant.
    pub value: AstExpr,
    /// Whether the item is public.
    pub is_pub: bool,
}

impl AstConstItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Const)?;
        let name = AstIdent::parse(lexer)?;
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

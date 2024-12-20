use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::AstIdent;
use shad_error::SyntaxError;

/// A parsed structure.
///
/// # Examples
///
/// The following Shad example will be parsed as a struct:
/// ```shad
/// struct Character {
///     life: f32,
///     energy: f32,
///     mana: f32,
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructItem {
    /// The struct name.
    pub name: AstIdent,
    /// The struct fields.
    pub fields: Vec<AstStructField>,
}

impl AstStructItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Struct)?;
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let mut fields = vec![];
        let mut is_first_field = true;
        while is_first_field || parse_token_option(lexer, TokenType::CloseBrace)?.is_none() {
            fields.push(AstStructField::parse(lexer)?);
            is_first_field = false;
            if parse_token_option(lexer, TokenType::Comma)?.is_none() {
                parse_token(lexer, TokenType::CloseBrace)?;
                break;
            }
        }
        Ok(Self { name, fields })
    }
}

/// A parsed struct field.
///
/// # Examples
///
/// `life: f32` is parsed as a field in the following Shad example:
/// ```shad
/// struct Character {
///     life: f32,
///     energy: f32,
///     mana: f32,
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructField {
    /// The field name.
    pub name: AstIdent,
    /// The field type.
    pub type_: AstIdent,
}

impl AstStructField {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let type_ = AstIdent::parse(lexer)?;
        Ok(Self { name, type_ })
    }
}

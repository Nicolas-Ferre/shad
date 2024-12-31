use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::AstIdent;
use shad_error::SyntaxError;

/// The parsed generic parameters of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstItemGenerics {
    /// The generics parameters.
    pub params: Vec<AstItemGenericParam>,
}

impl AstItemGenerics {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        if parse_token_option(lexer, TokenType::OpenAngleBracket)?.is_some() {
            let mut params = vec![AstItemGenericParam::parse(lexer)?];
            while parse_token_option(lexer, TokenType::Comma)?.is_some() {
                if parse_token(&mut lexer.clone(), TokenType::CloseAngleBracket).is_ok() {
                    break;
                }
                params.push(AstItemGenericParam::parse(lexer)?);
            }
            parse_token(lexer, TokenType::CloseAngleBracket)?;
            Ok(Self { params })
        } else {
            Ok(Self { params: vec![] })
        }
    }
}

/// A parsed generic parameter of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstItemGenericParam {
    /// The parameter name.
    pub name: AstIdent,
    /// The parameter type.
    pub type_: Option<AstIdent>,
}

impl AstItemGenericParam {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        let type_ = if parse_token_option(lexer, TokenType::Colon)?.is_some() {
            Some(AstIdent::parse(lexer)?)
        } else {
            None
        };
        Ok(Self { name, type_ })
    }
}

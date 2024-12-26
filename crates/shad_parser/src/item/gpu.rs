use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::AstIdent;
use shad_error::SyntaxError;

/// A parsed `gpu` qualifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGpuQualifier {
    /// The name of the item in WGSL.
    pub name: Option<AstIdent>,
}

impl AstGpuQualifier {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if parse_token_option(lexer, TokenType::Gpu)?.is_some() {
            if parse_token_option(lexer, TokenType::OpenParenthesis)?.is_some() {
                let name = AstIdent::parse(lexer)?;
                parse_token(lexer, TokenType::CloseParenthesis)?;
                Ok(Some(Self { name: Some(name) }))
            } else {
                Ok(Some(Self { name: None }))
            }
        } else {
            Ok(None)
        }
    }
}

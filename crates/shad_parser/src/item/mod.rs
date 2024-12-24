use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstStatement, AstStructItem};
use buffer::AstBufferItem;
use function::AstFnItem;
use import::AstImportItem;
use run_block::AstRunItem;
use shad_error::SyntaxError;

pub(crate) mod buffer;
pub(crate) mod function;
pub(crate) mod import;
pub(crate) mod run_block;
pub(crate) mod struct_;

/// A parsed item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstItem {
    /// A struct definition.
    Struct(AstStructItem),
    /// A buffer definition.
    Buffer(AstBufferItem),
    /// A function definition.
    Fn(AstFnItem),
    /// A run block.
    Run(AstRunItem),
    /// An imported module.
    Import(AstImportItem),
}

impl AstItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let token = lexer.clone().next_token()?;
        if token.type_ == TokenType::Struct {
            Ok(Self::Struct(AstStructItem::parse(lexer)?))
        } else {
            if token.type_ == TokenType::Pub {
                parse_token(lexer, TokenType::Pub)?;
            }
            Self::parse_without_visibility(lexer, token.type_ == TokenType::Pub)
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_without_visibility(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let token = lexer.clone().next_token()?;
        match token.type_ {
            TokenType::Buf => Ok(Self::Buffer(AstBufferItem::parse(lexer, is_pub)?)),
            TokenType::Gpu => Ok(Self::Fn(AstFnItem::parse_gpu(lexer, is_pub)?)),
            TokenType::Fn => Ok(Self::Fn(AstFnItem::parse(lexer, is_pub)?)),
            TokenType::Run => Ok(Self::Run(AstRunItem::parse(lexer)?)),
            TokenType::Import => Ok(Self::Import(AstImportItem::parse(lexer, is_pub)?)),
            _ => Err(SyntaxError::new(
                token.span.start,
                lexer.module(),
                "expected item",
            )),
        }
    }
}

fn parse_statement_block(lexer: &mut Lexer<'_>) -> Result<Vec<AstStatement>, SyntaxError> {
    parse_token(lexer, TokenType::OpenBrace)?;
    let mut statements = vec![];
    while parse_token_option(lexer, TokenType::CloseBrace)?.is_none() {
        statements.push(AstStatement::parse(lexer)?);
    }
    Ok(statements)
}

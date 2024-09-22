use crate::atom::parse_token;
use crate::token::{Token, TokenType};
use crate::{AstExpr, AstIdent, AstStatement};
use logos::Lexer;
use shad_error::SyntaxError;

/// A parsed item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstItem {
    /// A buffer definition.
    Buffer(AstBufferItem),
    /// A GPU function definition.
    GpuFn(AstGpuFnItem),
    /// A run block.
    Run(AstRunItem),
}

impl AstItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Buf => Ok(Self::Buffer(AstBufferItem::parse(lexer)?)),
            TokenType::Gpu => Ok(Self::GpuFn(AstGpuFnItem::parse(lexer)?)),
            TokenType::Run => Ok(Self::Run(AstRunItem::parse(lexer)?)),
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
    /// The name of the buffer.
    pub name: AstIdent,
    /// The initial value of the buffer.
    pub value: AstExpr,
}

impl AstBufferItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Buf)?;
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let value = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self { name, value })
    }
}

/// A parsed GPU function definition.
///
/// # Examples
///
/// Shad code `gpu fn sqrt(value: f32) -> f32;` will be parsed as a GPU function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGpuFnItem {
    /// The name of the function.
    pub name: AstIdent,
    /// The parameters of the function.
    pub params: Vec<AstFnParam>,
    /// The return type of the function.
    pub return_type: AstIdent,
}

impl AstGpuFnItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Gpu)?;
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer)?;
        let params = Self::parse_params(lexer)?;
        parse_token(lexer, TokenType::Arrow)?;
        let return_type = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            name,
            params,
            return_type,
        })
    }

    fn parse_params(lexer: &mut Lexer<'_, TokenType>) -> Result<Vec<AstFnParam>, SyntaxError> {
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut params = vec![];
        while parse_token(&mut lexer.clone(), TokenType::CloseParenthesis).is_err() {
            params.push(AstFnParam::parse(lexer)?);
            if parse_token(&mut lexer.clone(), TokenType::Comma).is_ok() {
                parse_token(lexer, TokenType::Comma)?;
            }
        }
        parse_token(lexer, TokenType::CloseParenthesis)?;
        Ok(params)
    }
}

/// A parsed function parameter.
///
/// # Examples
///
/// In Shad code `gpu fn sqrt(value: f32) -> f32;`, `value: f32` will be parsed as a function param.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnParam {
    /// The name of the parameter.
    pub name: AstIdent,
    /// The type of the parameter.
    pub type_: AstIdent,
}

impl AstFnParam {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let type_ = AstIdent::parse(lexer)?;
        Ok(Self { name, type_ })
    }
}

/// A parsed run block.
///
/// # Examples
///
/// In Shad code `run { my_buffer = 2.; }` will be parsed as a run block.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstRunItem {
    /// The statements inside the block.
    pub statements: Vec<AstStatement>,
}

impl AstRunItem {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Run)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let mut statements = vec![];
        while parse_token(&mut lexer.clone(), TokenType::CloseBrace).is_err() {
            statements.push(AstStatement::parse(lexer)?);
        }
        parse_token(lexer, TokenType::CloseBrace)?;
        Ok(Self { statements })
    }
}

use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstIdent, AstStatement};
use shad_error::{Span, SyntaxError};

/// A parsed GPU function definition.
///
/// # Examples
///
/// Shad code `gpu fn sqrt(value: f32) -> f32;` will be parsed as a GPU function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnItem {
    /// The function name.
    pub name: AstIdent,
    /// The function parameters.
    pub params: Vec<AstFnParam>,
    /// The return type of the function.
    pub return_type: Option<AstReturnType>,
    /// The function body statements.
    pub statements: Vec<AstStatement>,
    /// Whether the item is public.
    pub is_pub: bool,
    /// Whether the item is imported from WGSL.
    pub is_gpu: bool,
}

impl AstFnItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        let statements = super::parse_statement_block(lexer)?;
        Ok(Self {
            name,
            params,
            return_type,
            statements,
            is_pub,
            is_gpu: false,
        })
    }

    pub(crate) fn parse_gpu(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Gpu)?;
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            name,
            params,
            return_type,
            statements: vec![],
            is_pub,
            is_gpu: true,
        })
    }

    fn parse_params(lexer: &mut Lexer<'_>) -> Result<Vec<AstFnParam>, SyntaxError> {
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut params = vec![];
        while parse_token_option(lexer, TokenType::CloseParenthesis)?.is_none() {
            params.push(AstFnParam::parse(lexer)?);
            if parse_token_option(lexer, TokenType::Comma)?.is_none() {
                parse_token(lexer, TokenType::CloseParenthesis)?;
                break;
            }
        }
        Ok(params)
    }
}

/// A parsed function return type.
///
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstReturnType {
    /// The type name.
    pub name: AstIdent,
    /// Whether the return type is a reference.
    pub is_ref: bool,
}

impl AstReturnType {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if parse_token_option(lexer, TokenType::Arrow)?.is_some() {
            let ref_span = parse_token_option(lexer, TokenType::Ref)?.map(|ref_| ref_.span);
            Ok(Some(Self {
                name: AstIdent::parse(lexer)?,
                is_ref: ref_span.is_some(),
            }))
        } else {
            Ok(None)
        }
    }
}

/// A parsed function parameter.
///
/// # Examples
///
/// In Shad code `gpu fn sqrt(value: f32) -> f32;`, `value: f32` will be parsed as a function param.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnParam {
    /// The name of the parameter.
    pub name: AstIdent,
    /// The type of the parameter.
    pub type_: AstIdent,
    /// Span of the `ref` keyword.
    pub ref_span: Option<Span>,
}

impl AstFnParam {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let ref_span = parse_token_option(lexer, TokenType::Ref)?.map(|ref_| ref_.span);
        let type_ = AstIdent::parse(lexer)?;
        Ok(Self {
            name,
            type_,
            ref_span,
        })
    }
}

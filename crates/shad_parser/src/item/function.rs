use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstGpuQualifier, AstIdent, AstItemGenerics, AstStatement, AstType};
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
    /// The function generic parameters.
    pub generics: AstItemGenerics,
    /// The function parameters.
    pub params: Vec<AstFnParam>,
    /// The return type of the function.
    pub return_type: Option<AstReturnType>,
    /// The function body statements.
    pub statements: Vec<AstStatement>,
    /// Whether the item is public.
    pub is_pub: bool,
    /// Whether the item is a `const` function.
    pub is_const: bool,
    /// The `gpu` qualifier.
    pub gpu_qualifier: Option<AstGpuQualifier>,
}

impl AstFnItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer)?;
        let generics = AstItemGenerics::parse(lexer)?;
        let open_parenthesis = parse_token(&mut lexer.clone(), TokenType::OpenParenthesis)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        let statements = super::parse_statement_block(lexer)?;
        Ok(Self {
            name,
            generics: generics.unwrap_or_else(|| AstItemGenerics {
                span: open_parenthesis.span,
                params: vec![],
            }),
            params,
            return_type,
            statements,
            is_pub,
            is_const: false,
            gpu_qualifier: None,
        })
    }

    pub(crate) fn parse_gpu(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let gpu_qualifier = AstGpuQualifier::parse(lexer)?;
        let is_const = parse_token_option(lexer, TokenType::Const)?.is_some();
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer)?;
        let generics = AstItemGenerics::parse(lexer)?;
        let open_parenthesis = parse_token(&mut lexer.clone(), TokenType::OpenParenthesis)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            name,
            generics: generics.unwrap_or_else(|| AstItemGenerics {
                span: open_parenthesis.span,
                params: vec![],
            }),
            params,
            return_type,
            statements: vec![],
            is_pub,
            is_const,
            gpu_qualifier,
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
    /// The type reference.
    pub type_: AstType,
    /// Whether the return type is a reference.
    pub is_ref: bool,
}

impl AstReturnType {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if parse_token_option(lexer, TokenType::Arrow)?.is_some() {
            let ref_span = parse_token_option(lexer, TokenType::Ref)?.map(|ref_| ref_.span);
            Ok(Some(Self {
                type_: AstType::parse(lexer)?,
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
    pub type_: AstType,
    /// Span of the `ref` keyword.
    pub ref_span: Option<Span>,
}

impl AstFnParam {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let ref_span = parse_token_option(lexer, TokenType::Ref)?.map(|ref_| ref_.span);
        let type_ = AstType::parse(lexer)?;
        Ok(Self {
            name,
            type_,
            ref_span,
        })
    }
}

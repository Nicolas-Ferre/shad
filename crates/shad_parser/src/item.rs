use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, Token, TokenType};
use crate::{AstExpr, AstIdent, AstIdentType, AstStatement};
use shad_error::{Span, SyntaxError};
use std::str::FromStr;

/// A parsed item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstItem {
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
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        if token.type_ == TokenType::Pub {
            parse_token(lexer, TokenType::Pub)?;
        }
        Self::parse_without_visibility(lexer, token.type_ == TokenType::Pub)
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_without_visibility(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Buf => Ok(Self::Buffer(AstBufferItem::parse(lexer, is_pub)?)),
            TokenType::Gpu => Ok(Self::Fn(AstFnItem::parse_gpu(lexer, is_pub)?)),
            TokenType::Fn => Ok(Self::Fn(AstFnItem::parse(lexer, is_pub)?)),
            TokenType::Run => Ok(Self::Run(AstRunItem::parse(lexer)?)),
            TokenType::Import => Ok(Self::Import(AstImportItem::parse(lexer, is_pub)?)),
            _ => Err(SyntaxError::new(
                token.span.start,
                lexer.module.clone(),
                "expected item",
            )),
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

/// A parsed GPU function definition.
///
/// # Examples
///
/// Shad code `gpu fn sqrt(value: f32) -> f32;` will be parsed as a GPU function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnItem {
    /// The name of the function.
    pub name: AstIdent,
    /// The parameters of the function.
    pub params: Vec<AstFnParam>,
    /// The return type of the function.
    pub return_type: Option<AstReturnType>,
    /// The qualifier of the function.
    pub qualifier: AstFnQualifier,
    /// The qualifier of the function.
    pub statements: Vec<AstStatement>,
    /// Whether the item is public.
    pub is_pub: bool,
}

impl AstFnItem {
    fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer, AstIdentType::FnDef)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        let statements = parse_statement_block(lexer)?;
        Ok(Self {
            name,
            params,
            return_type,
            qualifier: AstFnQualifier::None,
            statements,
            is_pub,
        })
    }

    fn parse_gpu(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Gpu)?;
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer, AstIdentType::FnDef)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            name,
            params,
            return_type,
            qualifier: AstFnQualifier::Gpu,
            statements: vec![],
            is_pub,
        })
    }

    fn parse_params(lexer: &mut Lexer<'_>) -> Result<Vec<AstFnParam>, SyntaxError> {
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut params = vec![];
        while parse_token_option(lexer, TokenType::CloseParenthesis)?.is_none() {
            params.push(AstFnParam::parse(lexer)?);
            parse_token_option(lexer, TokenType::Comma)?;
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
                name: AstIdent::parse(lexer, AstIdentType::TypeUsage)?,
                is_ref: ref_span.is_some(),
            }))
        } else {
            Ok(None)
        }
    }
}

/// A parsed function qualifier.
///
/// A qualifier is a keyword that is placed before the `fn` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstFnQualifier {
    /// No qualifier.
    None,
    /// The `gpu` qualifier.
    Gpu,
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
        let ref_span = parse_token_option(lexer, TokenType::Ref)?.map(|ref_| ref_.span);
        let name = AstIdent::parse(lexer, AstIdentType::ParamDef)?;
        parse_token(lexer, TokenType::Colon)?;
        let type_ = AstIdent::parse(lexer, AstIdentType::TypeUsage)?;
        Ok(Self {
            name,
            type_,
            ref_span,
        })
    }
}

/// A parsed run block.
///
/// # Examples
///
/// The following Shad examples will be parsed as a run block:
/// - `run { my_buffer = 2.; }`
/// - `run priority 10 { my_buffer = 2.; }`
/// - `run priority -42 { my_buffer = 2.; }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstRunItem {
    /// The statements inside the block.
    pub statements: Vec<AstStatement>,
    /// The execution priority.
    pub priority: Option<i32>,
    /// The unique ID of the `run` block.
    pub id: u64,
}

impl AstRunItem {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Run)?;
        let priority = Self::parse_priority(lexer)?;
        let statements = parse_statement_block(lexer)?;
        Ok(Self {
            statements,
            priority,
            id: lexer.next_id(),
        })
    }

    fn parse_priority(lexer: &mut Lexer<'_>) -> Result<Option<i32>, SyntaxError> {
        Ok(
            if parse_token_option(lexer, TokenType::Priority)?.is_some() {
                Some(Self::parse_priority_value(lexer)?)
            } else {
                None
            },
        )
    }

    fn parse_priority_value(lexer: &mut Lexer<'_>) -> Result<i32, SyntaxError> {
        let is_neg = parse_token_option(lexer, TokenType::Minus)?.is_some();
        let value = parse_token(lexer, TokenType::I32Literal)?;
        i32::from_str(&value.slice.replace('_', ""))
            .map(|value| if is_neg { -value } else { value })
            .map_err(|_| {
                SyntaxError::new(
                    value.span.start,
                    lexer.module.clone(),
                    "`i32` literal out of range".to_string(),
                )
            })
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

/// A parsed import item.
///
/// # Examples
///
/// In Shad code `import a.b.c;` will be parsed as an import item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstImportItem {
    /// Path segments of the imported module.
    pub segments: Vec<AstIdent>,
    /// The span of the item.
    pub span: Span,
    /// Whether the item is public.
    pub is_pub: bool,
}

impl AstImportItem {
    fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let import = parse_token(lexer, TokenType::Import)?;
        let mut segments = vec![AstIdent::parse(lexer, AstIdentType::ModPathSegment)?];
        while parse_token_option(lexer, TokenType::Dot)?.is_some() {
            segments.push(AstIdent::parse(lexer, AstIdentType::ModPathSegment)?);
        }
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&import.span, &semi_colon.span),
            segments,
            is_pub,
        })
    }
}

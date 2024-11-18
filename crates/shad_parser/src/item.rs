use crate::atom::parse_token;
use crate::token::{Lexer, Token, TokenType};
use crate::{AstExpr, AstIdent, AstIdentType, AstStatement};
use shad_error::{Span, SyntaxError};

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
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        let next_token = Token::next(&mut tmp_lexer)?;
        match token.type_ {
            TokenType::Buf => {
                if next_token.type_ == TokenType::Fn {
                    Ok(Self::Fn(AstFnItem::parse(lexer, is_pub)?))
                } else {
                    Ok(Self::Buffer(AstBufferItem::parse(lexer, is_pub)?))
                }
            }
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
        let is_buf_qualifier = if parse_token(&mut lexer.clone(), TokenType::Fn).is_ok() {
            false
        } else {
            parse_token(lexer, TokenType::Buf)?;
            true
        };
        parse_token(lexer, TokenType::Fn)?;
        let name = AstIdent::parse(lexer, AstIdentType::FnDef)?;
        let params = Self::parse_params(lexer)?;
        let return_type = AstReturnType::parse(lexer)?;
        let statements = parse_statement_block(lexer)?;
        Ok(Self {
            name,
            params,
            return_type,
            qualifier: if is_buf_qualifier {
                AstFnQualifier::Buf
            } else {
                AstFnQualifier::None
            },
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
        if parse_token(&mut lexer.clone(), TokenType::Arrow).is_ok() {
            parse_token(lexer, TokenType::Arrow)?;
            let ref_span = if parse_token(&mut lexer.clone(), TokenType::Ref).is_ok() {
                Some(parse_token(lexer, TokenType::Ref)?.span)
            } else {
                None
            };
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
    /// The `buf` qualifier.
    Buf,
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
        let ref_span = if parse_token(&mut lexer.clone(), TokenType::Ref).is_ok() {
            Some(parse_token(lexer, TokenType::Ref)?.span)
        } else {
            None
        };
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
/// In Shad code `run { my_buffer = 2.; }` will be parsed as a run block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstRunItem {
    /// The statements inside the block.
    pub statements: Vec<AstStatement>,
    /// The unique ID of the `run` block.
    pub id: u64,
}

impl AstRunItem {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Run)?;
        let statements = parse_statement_block(lexer)?;
        Ok(Self {
            statements,
            id: lexer.next_id(),
        })
    }
}

fn parse_statement_block(lexer: &mut Lexer<'_>) -> Result<Vec<AstStatement>, SyntaxError> {
    parse_token(lexer, TokenType::OpenBrace)?;
    let mut statements = vec![];
    while parse_token(&mut lexer.clone(), TokenType::CloseBrace).is_err() {
        statements.push(AstStatement::parse(lexer)?);
    }
    parse_token(lexer, TokenType::CloseBrace)?;
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
        while parse_token(&mut lexer.clone(), TokenType::Dot).is_ok() {
            parse_token(lexer, TokenType::Dot)?;
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

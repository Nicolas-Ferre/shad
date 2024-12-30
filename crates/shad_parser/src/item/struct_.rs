use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstGpuQualifier, AstIdent, AstItemGenerics};
use shad_error::{Span, SyntaxError};
use std::num::NonZeroU32;
use std::str::FromStr;

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
    /// The struct generic parameters.
    pub generics: AstItemGenerics,
    /// The struct fields.
    pub fields: Vec<AstStructField>,
    /// Whether the item is public.
    pub is_pub: bool,
    /// The `gpu` qualifier and associated parameters.
    pub gpu_qualifier: Option<AstGpuQualifier>,
    /// The struct layout.
    pub layout: Option<AstStructLayout>,
}

impl AstStructItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let gpu_qualifier = AstGpuQualifier::parse(lexer)?;
        let layout = AstStructLayout::parse(lexer)?;
        parse_token(lexer, TokenType::Struct)?;
        let name = AstIdent::parse(lexer)?;
        let generics = AstItemGenerics::parse(lexer)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let mut fields = vec![];
        while parse_token_option(lexer, TokenType::CloseBrace)?.is_none() {
            fields.push(AstStructField::parse(lexer)?);
            if parse_token_option(lexer, TokenType::Comma)?.is_none() {
                parse_token(lexer, TokenType::CloseBrace)?;
                break;
            }
        }
        Ok(Self {
            name,
            generics,
            fields,
            is_pub,
            gpu_qualifier,
            layout,
        })
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
    /// Whether the item is public.
    pub is_pub: bool,
}

impl AstStructField {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let is_pub = parse_token_option(lexer, TokenType::Pub)?.is_some();
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let type_ = AstIdent::parse(lexer)?;
        Ok(Self {
            name,
            type_,
            is_pub,
        })
    }
}

/// A parsed struct layout.
///
/// # Examples
///
/// `layout(12, 16)` is parsed with `size`=12 and `alignment`=16 in the following Shad example:
/// ```shad
/// gpu layout(12, 16) struct Character {
///     life: f32,
///     energy: f32,
///     mana: f32,
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructLayout {
    /// The span of the layout.
    pub span: Span,
    /// The struct memory size.
    pub size: NonZeroU32,
    /// The struct memory alignment.
    pub alignment: NonZeroU32,
}

impl AstStructLayout {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if let Some(layout) = parse_token_option(lexer, TokenType::Layout)? {
            parse_token(lexer, TokenType::OpenParenthesis)?;
            let size = Self::parse_value(lexer)?;
            parse_token(lexer, TokenType::Comma)?;
            let alignment = Self::parse_value(lexer)?;
            let close_parenthesis = parse_token(lexer, TokenType::CloseParenthesis)?;
            Ok(Some(Self {
                span: Span::join(&layout.span, &close_parenthesis.span),
                size,
                alignment,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_value(lexer: &mut Lexer<'_>) -> Result<NonZeroU32, SyntaxError> {
        let value = parse_token(lexer, TokenType::I32Literal)?;
        NonZeroU32::from_str(&value.slice.replace('_', "")).map_err(|_| {
            SyntaxError::new(
                value.span.start,
                lexer.module(),
                "non-zero `u32` literal out of range".to_string(),
            )
        })
    }
}

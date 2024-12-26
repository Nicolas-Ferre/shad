use crate::atom::{parse_token, parse_token_option};
use crate::expr::LITERALS;
use crate::token::{Lexer, TokenType};
use crate::{AstIdent, AstLiteral};
use shad_error::SyntaxError;

/// A parsed `gpu` qualifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGpuQualifier {
    /// The name of the item in WGSL.
    pub name: Option<AstGpuName>,
}

impl AstGpuQualifier {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if parse_token_option(lexer, TokenType::Gpu)?.is_some() {
            if parse_token_option(lexer, TokenType::OpenParenthesis)?.is_some() {
                let name = AstGpuName::parse(lexer)?;
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

/// The name of an item in WGSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGpuName {
    /// The root part of the item name.
    pub root: AstIdent,
    /// The generic parameters of the item name.
    pub generics: Vec<AstGpuGenericParam>,
}

impl AstGpuName {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        let generics = if parse_token_option(lexer, TokenType::OpenAngleBracket)?.is_some() {
            let mut generics = vec![AstGpuGenericParam::parse(lexer)?];
            while parse_token_option(lexer, TokenType::Comma)?.is_some() {
                if parse_token_option(&mut lexer.clone(), TokenType::CloseAngleBracket)?.is_some() {
                    break;
                }
                generics.push(AstGpuGenericParam::parse(lexer)?);
            }
            parse_token(lexer, TokenType::CloseAngleBracket)?;
            generics
        } else {
            vec![]
        };
        Ok(Self {
            root: name,
            generics,
        })
    }
}

/// A generic parameter of a WGSL item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstGpuGenericParam {
    /// An identifier.
    Ident(AstIdent),
    /// A literal.
    Literal(AstLiteral),
}

impl AstGpuGenericParam {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        if LITERALS.contains(&lexer.clone().next_token()?.type_) {
            AstLiteral::parse(lexer).map(AstGpuGenericParam::Literal)
        } else {
            AstIdent::parse(lexer).map(AstGpuGenericParam::Ident)
        }
    }
}

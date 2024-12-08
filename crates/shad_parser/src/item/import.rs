use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::AstIdent;
use shad_error::{Span, SyntaxError};

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
    pub(crate) fn parse(lexer: &mut Lexer<'_>, is_pub: bool) -> Result<Self, SyntaxError> {
        let import = parse_token(lexer, TokenType::Import)?;
        let mut segments = vec![AstIdent::parse(lexer)?];
        while parse_token_option(lexer, TokenType::Dot)?.is_some() {
            segments.push(AstIdent::parse(lexer)?);
        }
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&import.span, &semi_colon.span),
            segments,
            is_pub,
        })
    }
}

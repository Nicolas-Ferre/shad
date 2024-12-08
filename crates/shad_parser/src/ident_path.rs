use crate::atom::parse_token_option;
use crate::token::{Lexer, TokenType};
use crate::AstIdent;
use shad_error::{Span, SyntaxError};

/// A parsed identifier path.
///
/// # Examples
///
/// The following Shad expressions will be parsed as an identifier path:
/// - `my_var`
/// - `my_var.field`
/// - `my_var.field.subfield.subsubfield`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstIdentPath {
    /// The path segments.
    pub segments: Vec<AstIdent>,
    /// The span of the identifier path.
    pub span: Span,
}

impl AstIdentPath {
    /// Replaces the first path segment by another path.
    pub fn replace_first_segment(&mut self, replacement: &Self) {
        let tail = self.segments.iter().skip(1).cloned().collect();
        self.segments = [replacement.segments.clone(), tail].concat();
    }

    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut segments = vec![AstIdent::parse(lexer)?];
        while parse_token_option(lexer, TokenType::Dot)?.is_some() {
            segments.push(AstIdent::parse(lexer)?);
        }
        Ok(Self {
            span: Span::join(&segments[0].span, &segments[segments.len() - 1].span),
            segments,
        })
    }
}

impl From<AstIdent> for AstIdentPath {
    fn from(ident: AstIdent) -> Self {
        Self {
            span: ident.span.clone(),
            segments: vec![ident],
        }
    }
}

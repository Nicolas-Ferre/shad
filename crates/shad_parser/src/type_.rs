use crate::token::Lexer;
use crate::{AstGenerics, AstIdent};
use shad_error::{Span, SyntaxError};

/// A parsed type reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstType {
    /// The span of the type reference.
    pub span: Span,
    /// The type name.
    pub name: AstIdent,
    /// The type generic arguments.
    pub generics: AstGenerics,
}

impl From<AstIdent> for AstType {
    fn from(name: AstIdent) -> Self {
        Self {
            span: name.span.clone(),
            generics: AstGenerics {
                span: Span::new(name.span.end - 2, name.span.end, name.span.module.clone()),
                args: vec![],
            },
            name,
        }
    }
}

impl AstType {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        let generics = AstGenerics::parse(lexer)?.unwrap_or_else(|| AstGenerics {
            span: Span::new(name.span.end - 2, name.span.end, name.span.module.clone()),
            args: vec![],
        });
        Ok(Self {
            span: Span::join(&name.span, &generics.span),
            generics,
            name,
        })
    }
}

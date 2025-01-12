use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::AstExpr;
use shad_error::{Span, SyntaxError};

/// Parsed generic arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGenerics {
    /// The span of the generics.
    pub span: Span,
    /// The generic arguments.
    pub args: Vec<AstExpr>,
}

impl AstGenerics {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if let Some(open_bracket) = parse_token_option(lexer, TokenType::OpenAngleBracket)? {
            let mut args = vec![AstExpr::parse(lexer, true)?];
            while parse_token_option(lexer, TokenType::Comma)?.is_some() {
                if parse_token(&mut lexer.clone(), TokenType::CloseAngleBracket).is_ok() {
                    break;
                }
                args.push(AstExpr::parse(lexer, true)?);
            }
            let close_bracket = parse_token(lexer, TokenType::CloseAngleBracket)?;
            for arg in &mut args {
                arg.is_const = true;
            }
            Ok(Some(Self {
                span: Span::join(&open_bracket.span, &close_bracket.span),
                args,
            }))
        } else {
            Ok(None)
        }
    }
}

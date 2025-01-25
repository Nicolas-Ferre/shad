use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstExpr, AstType};
use shad_error::{Span, SyntaxError};

/// Parsed generic arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGenerics {
    /// The span of the generics.
    pub span: Span,
    /// The generic arguments.
    pub args: Vec<AstGenericArg>,
}

impl AstGenerics {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Option<Self>, SyntaxError> {
        if let Some(open_bracket) = parse_token_option(lexer, TokenType::OpenAngleBracket)? {
            let mut args = vec![AstGenericArg::parse(lexer)?];
            while parse_token_option(lexer, TokenType::Comma)?.is_some() {
                if parse_token(&mut lexer.clone(), TokenType::CloseAngleBracket).is_ok() {
                    break;
                }
                args.push(AstGenericArg::parse(lexer)?);
            }
            let close_bracket = parse_token(lexer, TokenType::CloseAngleBracket)?;
            for arg in &mut args {
                if let AstGenericArg::Expr(arg) = arg {
                    arg.is_const = true;
                }
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

/// A parsed generic argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstGenericArg {
    /// An expression.
    Expr(AstExpr),
    /// A type reference.
    Type(AstType),
}

impl AstGenericArg {
    /// Returns the span of the expression.
    pub fn span(&self) -> &Span {
        match self {
            Self::Expr(expr) => &expr.span,
            Self::Type(type_) => &type_.span,
        }
    }

    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        if AstExpr::parse(&mut lexer.clone(), true).is_ok() {
            Ok(Self::Expr(AstExpr::parse(lexer, true)?))
        } else {
            Ok(Self::Type(AstType::parse(lexer)?))
        }
    }
}

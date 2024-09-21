use crate::atom::parse_token;
use crate::common::{Token, TokenType};
use crate::{AstIdent, AstLiteral};
use logos::Lexer;
use shad_error::{Span, SyntaxError};

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpr {
    /// A literal.
    Literal(AstLiteral),
    /// An identifier.
    Ident(AstIdent),
    /// A function call.
    FnCall(AstFnCall),
}

impl AstExpr {
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(expr) => expr.span,
            Self::Ident(expr) => expr.span,
            Self::FnCall(expr) => expr.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        let next_token = Token::next(&mut tmp_lexer)?;
        match token.type_ {
            type_ @ (TokenType::F32Literal
            | TokenType::U32Literal
            | TokenType::I32Literal
            | TokenType::True
            | TokenType::False) => Ok(Self::Literal(AstLiteral::parse(lexer, type_)?)),
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(AstFnCall::parse(lexer)?))
                } else {
                    Ok(Self::Ident(AstIdent::parse(lexer)?))
                }
            }
            _ => Err(SyntaxError::new(token.span.start, "expected expression")),
        }
    }
}

/// A parsed function call.
///
/// This corresponds to the identifier of the function, followed between parentheses by
/// comma-separated arguments.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a function call:
/// - `myfunc()`
/// - `myfunc(expr)`
/// - `myfunc(expr1, expr2)`
/// - `myfunc(expr1, expr2,)`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnCall {
    /// The span of the function call.
    pub span: Span,
    /// The function name.
    pub name: AstIdent,
    /// The arguments passed to the function.
    pub args: Vec<AstExpr>,
}

impl AstFnCall {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut args = vec![];
        while parse_token(&mut lexer.clone(), TokenType::CloseParenthesis).is_err() {
            args.push(AstExpr::parse(lexer)?);
            if parse_token(&mut lexer.clone(), TokenType::Comma).is_ok() {
                parse_token(lexer, TokenType::Comma)?;
            }
        }
        let close_parenthesis = parse_token(lexer, TokenType::CloseParenthesis)?;
        Ok(Self {
            span: Span::new(name.span.start, close_parenthesis.span.end),
            name,
            args,
        })
    }
}

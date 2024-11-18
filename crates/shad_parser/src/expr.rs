use crate::atom::parse_token;
use crate::fn_call::AstFnCall;
use crate::token::{Lexer, Token, TokenType};
use crate::{AstIdent, AstIdentType, AstLiteral};
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
    pub fn span(&self) -> &Span {
        match self {
            Self::Literal(expr) => &expr.span,
            Self::Ident(expr) => &expr.span,
            Self::FnCall(expr) => &expr.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut expressions = vec![Self::parse_part(lexer)?];
        let mut operators = vec![];
        loop {
            let token = Token::next(&mut lexer.clone())?;
            if [
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Star,
                TokenType::Slash,
                TokenType::Percent,
                TokenType::Eq,
                TokenType::NotEq,
                TokenType::GreaterThanOrEq,
                TokenType::LessThanOrEq,
                TokenType::OpenAngleBracket,
                TokenType::CloseAngleBracket,
                TokenType::And,
                TokenType::Or,
            ]
            .contains(&token.type_)
            {
                operators.push((token.type_, token.span));
            } else {
                break;
            }
            let _operator = Token::next(lexer)?;
            expressions.push(Self::parse_part(lexer)?);
        }
        if expressions.len() == 1 {
            Ok(expressions.remove(0))
        } else {
            AstFnCall::parse_binary_operation(lexer, &expressions, &operators).map(AstExpr::FnCall)
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse_part(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        let next_token = Token::next(&mut tmp_lexer)?;
        match token.type_ {
            TokenType::OpenParenthesis => {
                parse_token(lexer, TokenType::OpenParenthesis)?;
                let expr = Self::parse(lexer)?;
                parse_token(lexer, TokenType::CloseParenthesis)?;
                Ok(expr)
            }
            type_ @ (TokenType::F32Literal
            | TokenType::U32Literal
            | TokenType::I32Literal
            | TokenType::True
            | TokenType::False) => Ok(Self::Literal(AstLiteral::parse(lexer, type_)?)),
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(AstFnCall::parse(lexer, false)?))
                } else {
                    Ok(Self::Ident(AstIdent::parse(lexer, AstIdentType::VarUsage)?))
                }
            }
            TokenType::Minus => Ok(Self::FnCall(AstFnCall::parse_unary_operation(lexer)?)),
            TokenType::Not => Ok(Self::FnCall(AstFnCall::parse_unary_operation(lexer)?)),
            _ => Err(SyntaxError::new(
                token.span.start,
                lexer.module.clone(),
                "expected expression",
            )),
        }
    }
}

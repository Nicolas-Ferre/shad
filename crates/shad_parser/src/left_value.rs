use crate::fn_call::AstFnCall;
use crate::token::{Lexer, Token, TokenType};
use crate::{AstExpr, AstIdent, AstIdentType};
use shad_error::{Span, SyntaxError};

/// A parsed left value that can be assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstLeftValue {
    /// An identifier.
    Ident(AstIdent),
    /// A function call.
    FnCall(AstFnCall),
}

impl AstLeftValue {
    /// Returns the span of the value.
    pub fn span(&self) -> &Span {
        match self {
            Self::Ident(value) => &value.span,
            Self::FnCall(value) => &value.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let tmp_lexer = &mut lexer.clone();
        let token = Token::next(tmp_lexer)?;
        let next_token = Token::next(tmp_lexer)?;
        match token.type_ {
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(AstFnCall::parse(lexer, false)?))
                } else {
                    Ok(Self::Ident(AstIdent::parse(lexer, AstIdentType::VarUsage)?))
                }
            }
            _ => unreachable!("internal error: expected left value"),
        }
    }
}

impl TryFrom<AstExpr> for AstLeftValue {
    type Error = ();

    fn try_from(value: AstExpr) -> Result<Self, Self::Error> {
        match value {
            AstExpr::Ident(value) => Ok(Self::Ident(value)),
            AstExpr::FnCall(value) => Ok(Self::FnCall(value)), // no-coverage (never reached)
            AstExpr::Literal(_) => Err(()),                    // no-coverage (never reached)
        }
    }
}

impl From<AstLeftValue> for AstExpr {
    fn from(value: AstLeftValue) -> Self {
        match value {
            AstLeftValue::Ident(value) => Self::Ident(value),
            AstLeftValue::FnCall(value) => Self::FnCall(value),
        }
    }
}

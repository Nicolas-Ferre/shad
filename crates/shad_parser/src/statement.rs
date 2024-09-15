use crate::atom::parse_token;
use crate::common::TokenType;
use crate::{AstExpr, AstIdent};
use logos::Lexer;
use shad_error::SyntaxError;

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstStatement {
    /// An assignment statement.
    Assignment(AstAssignment),
}

impl AstStatement {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        Ok(Self::Assignment(AstAssignment::parse(lexer)?))
    }
}

/// An assignment.
///
/// # Examples
///
/// The Shad code `my_var = 2;` will be parsed as a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstAssignment {
    /// The updated value.
    pub value: AstIdent,
    /// The assigned expression.
    pub expr: AstExpr,
}

impl AstAssignment {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let value = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Equal)?;
        let expr = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self { value, expr })
    }
}

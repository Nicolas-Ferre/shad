use crate::atom::parse_token;
use crate::token::{Token, TokenType};
use crate::{AstExpr, AstIdent};
use logos::Lexer;
use shad_error::SyntaxError;

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstStatement {
    /// An assignment statement.
    Assignment(AstAssignment),
    /// A variable definition statement.
    Var(AstVarDefinition),
}

impl AstStatement {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Ident => Ok(Self::Assignment(AstAssignment::parse(lexer)?)),
            TokenType::Var => Ok(Self::Var(AstVarDefinition::parse(lexer)?)),
            _ => Err(SyntaxError::new(token.span.start, "expected statement")),
        }
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
        parse_token(lexer, TokenType::Assigment)?;
        let expr = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self { value, expr })
    }
}

/// A variable definition.
///
/// # Examples
///
/// The Shad code `var my_var = 2;` will be parsed as a variable definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstVarDefinition {
    /// The variable name.
    pub name: AstIdent,
    /// The initial value of the variable.
    pub expr: AstExpr,
}

impl AstVarDefinition {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Var)?;
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let expr = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self { name, expr })
    }
}

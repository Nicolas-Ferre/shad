use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstExpr, AstIdent};
use shad_error::{Span, SyntaxError};

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstStatement {
    /// An assignment statement.
    Assignment(AstAssignment),
    /// A variable definition statement.
    Var(AstVarDefinition),
    /// A return statement.
    Return(AstReturn),
    /// An expression.
    Expr(AstExprStatement),
}

impl AstStatement {
    // coverage: off (simple and not critical logic)
    /// Returns the span of the expression.
    pub fn span(&self) -> &Span {
        match self {
            Self::Assignment(statement) => &statement.span,
            Self::Var(statement) => &statement.span,
            Self::Return(statement) => &statement.span,
            Self::Expr(statement) => &statement.span,
        }
    }
    // coverage: on

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let token = lexer.clone().next_token()?;
        match token.type_ {
            TokenType::OpenParenthesis
            | TokenType::Ident
            | TokenType::F32Literal
            | TokenType::U32Literal
            | TokenType::I32Literal => {
                if AstExprStatement::parse(&mut lexer.clone()).is_ok() {
                    Ok(Self::Expr(AstExprStatement::parse(lexer)?))
                } else {
                    Ok(Self::Assignment(AstAssignment::parse(lexer)?))
                }
            }
            TokenType::Var | TokenType::Ref => Ok(Self::Var(AstVarDefinition::parse(lexer)?)),
            TokenType::Return => Ok(Self::Return(AstReturn::parse(lexer)?)),
            _ => Err(SyntaxError::new(
                token.span.start,
                lexer.module(),
                "expected statement",
            )),
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
    /// The span of the assignment.
    pub span: Span,
    /// The updated value.
    pub left: AstExpr,
    /// The assigned expression.
    pub right: AstExpr,
}

impl AstAssignment {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let left = AstExpr::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let right = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&left.span, &semi_colon.span),
            left,
            right,
        })
    }
}

/// A variable definition.
///
/// # Examples
///
/// Following Shad snippets will be parsed as a variable definition:
/// - `var my_var = 2;`
/// - `ref my_ref = other_var;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstVarDefinition {
    /// The span of the variable definition.
    pub span: Span,
    /// The variable name.
    pub name: AstIdent,
    /// Whether the variable is a reference.
    pub is_ref: bool,
    /// The initial value of the variable.
    pub expr: AstExpr,
}

impl AstVarDefinition {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let keyword = if let Some(var_) = parse_token_option(lexer, TokenType::Var)? {
            var_
        } else {
            parse_token(lexer, TokenType::Ref)?
        };
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&keyword.span, &semi_colon.span),
            name,
            is_ref: keyword.type_ == TokenType::Ref,
            expr,
        })
    }
}

/// A return statement.
///
/// # Examples
///
/// The Shad code `return 42;` will be parsed as a return statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstReturn {
    /// The span of the statement.
    pub span: Span,
    /// The returned expression.
    pub expr: AstExpr,
}

impl AstReturn {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let return_ = parse_token(lexer, TokenType::Return)?;
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&return_.span, &semi_colon.span),
            expr,
        })
    }
}

/// An expression statement.
///
/// # Examples
///
/// The Shad code `my_func(42);` will be parsed as an expression statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExprStatement {
    /// The span of the statement.
    pub span: Span,
    /// The expression.
    pub expr: AstExpr,
}

impl AstExprStatement {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(&expr.span, &semi_colon.span),
            expr,
        })
    }
}

use crate::atom::parse_token;
use crate::token::{Token, TokenType};
use crate::{AstExpr, AstFnCall, AstIdent, AstLeftValue};
use logos::Lexer;
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
    /// A function call.
    FnCall(AstFnCallStatement),
}

impl AstStatement {
    // coverage: off (simple and not critical logic)
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Assignment(statement) => statement.span,
            Self::Var(statement) => statement.span,
            Self::Return(statement) => statement.span,
            Self::FnCall(statement) => statement.span,
        }
    }
    // coverage: on

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let token = Token::next(&mut lexer.clone())?;
        match token.type_ {
            TokenType::Ident => {
                if AstFnCallStatement::parse(&mut lexer.clone()).is_ok() {
                    Ok(Self::FnCall(AstFnCallStatement::parse(lexer)?))
                } else {
                    Ok(Self::Assignment(AstAssignment::parse(lexer)?))
                }
            }
            TokenType::Var => Ok(Self::Var(AstVarDefinition::parse(lexer)?)),
            TokenType::Return => Ok(Self::Return(AstReturn::parse(lexer)?)),
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
    /// The span of the assignment.
    pub span: Span,
    /// The updated value.
    pub value: AstLeftValue,
    /// The assigned expression.
    pub expr: AstExpr,
}

impl AstAssignment {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let value = AstLeftValue::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(value.span(), semi_colon.span),
            value,
            expr,
        })
    }
}

/// A variable definition.
///
/// # Examples
///
/// The Shad code `var my_var = 2;` will be parsed as a variable definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstVarDefinition {
    /// The span of the variable definition.
    pub span: Span,
    /// The variable name.
    pub name: AstIdent,
    /// The initial value of the variable.
    pub expr: AstExpr,
}

impl AstVarDefinition {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let var_ = parse_token(lexer, TokenType::Var)?;
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::Assigment)?;
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(var_.span, semi_colon.span),
            name,
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
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let return_ = parse_token(lexer, TokenType::Return)?;
        let expr = AstExpr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(return_.span, semi_colon.span),
            expr,
        })
    }
}

/// A function call statement.
///
/// # Examples
///
/// The Shad code `my_func(42);` will be parsed as a function call statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnCallStatement {
    /// The span of the statement.
    pub span: Span,
    /// The function call.
    pub call: AstFnCall,
}

impl AstFnCallStatement {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let call = AstFnCall::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span::join(call.span, semi_colon.span),
            call,
        })
    }
}

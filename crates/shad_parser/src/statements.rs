use crate::atoms::parse_token;
use crate::common::{Token, TokenType};
use crate::{Expr, FnCall, Ident, ParsingError, Span, Value};
use logos::Lexer;

/// A parsed statement.
///
/// # Examples
///
/// - Shad code `let myvar = 42;` will be parsed as a [`Statement::Let`].
/// - Shad code `return myvar + 7;` will be parsed as a [`Statement::Return`].
/// - Shad code `for i in range(100) { print(i); }` will be parsed as a [`Statement::For`].
/// - Shad code `loop { print("Hello"); }` will be parsed as a [`Statement::Loop`].
/// - Shad code `myfunc(42);` will be parsed as a [`Statement::FnCall`].
/// - Shad code `myvar = square(myvar);` will be parsed as a [`Statement::Assignment`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// A `let` statement.
    Let(LetStatement),
    /// A `return` statement.
    Return(ReturnStatement),
    /// A `for` statement.
    For(ForStatement),
    /// A `loop` statement.
    Loop(LoopStatement),
    /// A function call statement.
    FnCall(FnCall),
    /// An variable assignment statement.
    Assignment(Assignment),
}

impl Statement {
    /// Returns the span of the statement.
    pub fn span(&self) -> Span {
        match self {
            Self::Let(statement) => statement.span,
            Self::Return(statement) => statement.span,
            Self::For(statement) => statement.span,
            Self::Loop(statement) => statement.span,
            Self::FnCall(statement) => statement.span,
            Self::Assignment(statement) => statement.span,
        }
    }

    pub(crate) fn parse_many(lexer: &mut Lexer<'_, TokenType>) -> Result<Vec<Self>, ParsingError> {
        let mut statements = vec![];
        while Token::next(&mut lexer.clone())?.type_ != TokenType::CloseBrace {
            statements.push(Self::parse(lexer)?);
        }
        Ok(statements)
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        let next_token = Token::next(&mut tmp_lexer)?;
        match token.type_ {
            TokenType::Let => Ok(Self::Let(LetStatement::parse(lexer)?)),
            TokenType::Return => Ok(Self::Return(ReturnStatement::parse(lexer)?)),
            TokenType::For => Ok(Self::For(ForStatement::parse(lexer)?)),
            TokenType::Loop => Ok(Self::Loop(LoopStatement::parse(lexer)?)),
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(Self::parse_fn_call(lexer)?))
                } else {
                    Ok(Self::Assignment(Assignment::parse(lexer)?))
                }
            }
            _ => Err(ParsingError::new(token.span.start, "expected statement")),
        }
    }

    fn parse_fn_call(lexer: &mut Lexer<'_, TokenType>) -> Result<FnCall, ParsingError> {
        let fn_call = FnCall::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        Ok(fn_call)
    }
}

/// A parsed `let` statement.
///
/// This statement is used to define a new variable.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a `let` statement:
/// - `let myvar = 42;`
/// - `let myvar = 2 * square(42);`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub span: Span,
    pub name: Ident,
    pub expr: Expr,
}

impl LetStatement {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let let_ = parse_token(lexer, TokenType::Let)?;
        let name = Ident::parse(lexer)?;
        parse_token(lexer, TokenType::Equal)?;
        let expr = Expr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span {
                start: let_.span.start,
                end: semi_colon.span.end,
            },
            name,
            expr,
        })
    }
}

/// A parsed `return` statement.
///
/// This statement is used to return an expression from a function.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a `let` statement:
/// - `return 42;`
/// - `return 2 * square(42);`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    /// The span.
    pub span: Span,
    /// The returned expression.
    pub expr: Expr,
}

impl ReturnStatement {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let return_ = parse_token(lexer, TokenType::Return)?;
        let expr = Expr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span {
                start: return_.span.start,
                end: semi_colon.span.end,
            },
            expr,
        })
    }
}

/// A parsed `for` loop.
///
/// This statement is used to loop on an iterable expression.
///
/// # Examples
///
/// The following Shad expression will be parsed as a `let` statement:
///
/// ```custom,rust
/// for i in range(100) {
///     let value = 2 * i;
///     result = result + value;
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStatement {
    /// The span.
    pub span: Span,
    /// The item variable identifier updated at each loop step.
    pub variable: Ident,
    /// The iterable expression.
    pub iterable: Expr,
    /// The statements inside the loop.
    pub statements: Vec<Statement>,
}

impl ForStatement {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let for_ = parse_token(lexer, TokenType::For)?;
        let variable = Ident::parse(lexer)?;
        parse_token(lexer, TokenType::In)?;
        let iterable = Expr::parse(lexer)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let statements = Statement::parse_many(lexer)?;
        let close_brace = parse_token(lexer, TokenType::CloseBrace)?;
        Ok(Self {
            span: Span {
                start: for_.span.start,
                end: close_brace.span.end,
            },
            variable,
            iterable,
            statements,
        })
    }
}

/// A parsed `loop` loop.
///
/// This statement is used to run the game loop, where the window rendering is refreshed at the
/// end of each step.
///
/// # Examples
///
/// The following Shad expression will be parsed as a `let` statement:
///
/// ```custom,rust
/// loop {
///     character.position = character.position + character.velocity * delta;
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStatement {
    /// The span.
    pub span: Span,
    /// The statements inside the loop.
    pub statements: Vec<Statement>,
}

impl LoopStatement {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let loop_ = parse_token(lexer, TokenType::Loop)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let statements = Statement::parse_many(lexer)?;
        let close_brace = parse_token(lexer, TokenType::CloseBrace)?;
        Ok(Self {
            span: Span {
                start: loop_.span.start,
                end: close_brace.span.end,
            },
            statements,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The span.
    pub span: Span,
    /// The updated value.
    pub value: Value,
    /// The assigned expression.
    pub expr: Expr,
}

impl Assignment {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let value = Value::parse(lexer)?;
        parse_token(lexer, TokenType::Equal)?;
        let expr = Expr::parse(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span {
                start: value.span.start,
                end: semi_colon.span.end,
            },
            value,
            expr,
        })
    }
}

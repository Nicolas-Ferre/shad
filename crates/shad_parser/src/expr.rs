use crate::atoms::parse_token;
use crate::common::{Token, TokenType};
use crate::{Ident, Literal, ParsingError, Span};
use logos::Lexer;

/// A parsed expression.
///
/// # Examples
///
/// - Shad code `-42` will be parsed as an [`Expr::UnarySub`].
/// - Shad code `a + 3` will be parsed as an [`Expr::BinaryOp`].
/// - Shad code `[item_init(); 100]` will be parsed as an [`Expr::Array`].
/// - Shad code `myfunc(a, 42)` will be parsed as an [`Expr::FnCall`].
/// - Shad code `characters[0].position` will be parsed as an [`Expr::Value`].
/// - Shad code `3.14` will be parsed as an [`Expr::Literal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A unary subtraction.
    UnarySub(UnarySub),
    /// A binary operation.
    BinaryOp(BinaryOp),
    /// An array initialization.
    Array(Array),
    /// A function call.
    FnCall(FnCall),
    /// A value.
    Value(Value),
    /// A literal.
    Literal(Literal),
}

impl Expr {
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Self::UnarySub(expr) => expr.span,
            Self::BinaryOp(expr) => expr.span,
            Self::Array(expr) => expr.span,
            Self::FnCall(expr) => expr.span,
            Self::Value(expr) => expr.span,
            Self::Literal(expr) => expr.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let mut expressions = vec![Self::parse_part(lexer)?];
        let mut operators = vec![];
        loop {
            match Token::next(&mut lexer.clone())?.type_ {
                TokenType::Add => operators.push(BinaryOperator::Add),
                TokenType::Sub => operators.push(BinaryOperator::Sub),
                TokenType::Mul => operators.push(BinaryOperator::Mul),
                TokenType::Div => operators.push(BinaryOperator::Div),
                _ => break,
            }
            let _operator = Token::next(lexer)?;
            expressions.push(Self::parse_part(lexer)?);
        }
        if expressions.len() == 1 {
            Ok(expressions.remove(0))
        } else {
            BinaryOp::parse(&expressions, &operators).map(Expr::BinaryOp)
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_part(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let mut tmp_lexer = lexer.clone();
        let token = Token::next(&mut tmp_lexer)?;
        let next_token = Token::next(&mut tmp_lexer)?;
        match token.type_ {
            TokenType::OpenParenthesis => {
                let expr = Self::parse(lexer)?;
                parse_token(lexer, TokenType::CloseParenthesis)?;
                Ok(expr)
            }
            TokenType::Ident => {
                if next_token.type_ == TokenType::OpenParenthesis {
                    Ok(Self::FnCall(FnCall::parse(lexer)?))
                } else {
                    Ok(Self::Value(Value::parse(lexer)?))
                }
            }
            TokenType::FloatLiteral => Ok(Self::Literal(Literal::parse_float(lexer)?)),
            TokenType::IntLiteral => Ok(Self::Literal(Literal::parse_int(lexer)?)),
            TokenType::Sub => Ok(Self::UnarySub(UnarySub::parse(lexer)?)),
            TokenType::OpenSquareBracket => Ok(Self::Array(Array::parse(lexer)?)),
            _ => Err(ParsingError::new(token.span.start, "expected expression")),
        }
    }
}

/// A parsed unary subtraction.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a unary subtraction:
/// - `-42`
/// - `-value`
/// - `-square(value)`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnarySub {
    pub span: Span,
    pub expr: Box<Expr>,
}

impl UnarySub {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let sub = parse_token(lexer, TokenType::Sub)?;
        let expr = Expr::parse_part(lexer)?;
        Ok(Self {
            span: Span {
                start: sub.span.start,
                end: expr.span().end,
            },
            expr: Box::new(expr),
        })
    }
}

/// A parsed binary operation.
///
/// In case no parenthesis is used to enforce an operator priority, the following priority order is
/// applied:
/// - `*` and `/`
/// - `+` and `-`
///
/// # Examples
///
/// The following Shad expressions will be parsed as a binary operation:
/// - `2 + 3`
/// - `2 + 3 * 5 - 12` (in this example, `+` is taken as the highest operator in [`BinaryOp`])
/// - `1 - value`
/// - `2 * square(value)`
/// - `(3+5) * (7-8)`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub span: Span,
    pub left: Box<Expr>,
    pub operator: BinaryOperator,
    pub right: Box<Expr>,
}

impl BinaryOp {
    fn parse(expressions: &[Expr], operators: &[BinaryOperator]) -> Result<Self, ParsingError> {
        let operator_priority = [
            vec![BinaryOperator::Mul, BinaryOperator::Div],
            vec![BinaryOperator::Add, BinaryOperator::Sub],
        ];
        let operator_index = operator_priority
            .iter()
            .rev()
            .find_map(|ops| operators.iter().position(|op| ops.contains(op)))
            .expect("expected binary operator");
        let left = if operator_index == 0 {
            Box::new(expressions[0].clone())
        } else {
            Box::new(Expr::BinaryOp(Self::parse(
                &expressions[..=operator_index],
                &operators[..operator_index],
            )?))
        };
        let right = if operator_index == operators.len() - 1 {
            Box::new(expressions[expressions.len() - 1].clone())
        } else {
            Box::new(Expr::BinaryOp(Self::parse(
                &expressions[operator_index + 1..],
                &operators[operator_index + 1..],
            )?))
        };
        Ok(Self {
            span: Span {
                start: left.span().start,
                end: right.span().end,
            },
            left,
            operator: operators[operator_index],
            right,
        })
    }
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// `+` operator.
    Add,
    /// `-` operator.
    Sub,
    /// `*` operator.
    Mul,
    /// `/` operator.
    Div,
}

/// A parsed array initialization.
///
/// An array initialization structured as `[init; size]`, where `init` is an expression that
/// generates the value of each item of the array, and `size` an expression that gives the
/// size of the array in `int`.
///
/// # Examples
///
/// The following Shad expressions will be parsed as an array initialization:
/// - `[0.; 100]`
/// - `[random_int(); dynamic_size()]`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    /// The span.
    pub span: Span,
    /// The expression that generates each item of the array.
    pub item: Box<Expr>,
    /// The expression that gives the array size.
    pub size: Box<Expr>,
}

impl Array {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let open_bracket = parse_token(lexer, TokenType::OpenSquareBracket)?;
        let item = Expr::parse(lexer)?;
        parse_token(lexer, TokenType::SemiColon)?;
        let size = Expr::parse(lexer)?;
        let close_bracket = parse_token(lexer, TokenType::CloseSquareBracket)?;
        Ok(Self {
            span: Span {
                start: open_bracket.span.start,
                end: close_bracket.span.end,
            },
            item: Box::new(item),
            size: Box::new(size),
        })
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
pub struct FnCall {
    /// The span.
    pub span: Span,
    /// The function name.
    pub name: Ident,
    /// The arguments passed to the function.
    pub args: Vec<Expr>,
}

impl FnCall {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let name = Ident::parse(lexer)?;
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut args = vec![];
        loop {
            let token = Token::next(&mut lexer.clone())?;
            if token.type_ == TokenType::CloseParenthesis {
                break;
            }
            args.push(Expr::parse(lexer)?);
            let token = Token::next(&mut lexer.clone())?;
            if token.type_ == TokenType::Comma {
                Token::next(lexer)?;
            } else if token.type_ != TokenType::CloseParenthesis {
                break;
            }
        }
        let close_parenthesis = parse_token(lexer, TokenType::CloseParenthesis)?;
        Ok(Self {
            span: Span {
                start: name.span.start,
                end: close_parenthesis.span.end,
            },
            name,
            args,
        })
    }
}

/// A parsed value.
///
/// A value is a variable identifier optionally following by field chaining separated by `.`.
/// Each part of the value can be indexed with `[x]` syntax to access a specific value from an
/// array.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a value:
/// - `myvar`
/// - `myvar.field`
/// - `myvar.field.otherfield`
/// - `myvar[0].field[2].otherfield`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    /// The span.
    pub span: Span,
    /// The parts of the value.
    pub segments: Vec<ValueSegment>,
}

impl Value {
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let mut segments = vec![ValueSegment::parse(lexer)?];
        while Token::next(&mut lexer.clone())?.type_ == TokenType::Dot {
            parse_token(lexer, TokenType::Dot)?;
            segments.push(ValueSegment::parse(lexer)?);
        }
        Ok(Self {
            span: Span {
                start: segments[0].span.start,
                end: segments[segments.len() - 1].span.end,
            },
            segments,
        })
    }
}

/// A parsed value segment.
///
/// A value segment is a part of a [`Value`], delimited by `.`.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a value segment:
/// - `field`
/// - `field[2]`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueSegment {
    /// The span.
    pub span: Span,
    /// The name of the variable of field included in the segment.
    pub name: Ident,
    /// The index used to access a specific item of an array.
    pub index: Option<Box<Expr>>,
}

impl ValueSegment {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let name = Ident::parse(lexer)?;
        if Token::next(&mut lexer.clone())?.type_ == TokenType::OpenSquareBracket {
            parse_token(lexer, TokenType::OpenSquareBracket)?;
            let index = Expr::parse(lexer)?;
            let close_bracket = parse_token(lexer, TokenType::CloseSquareBracket)?;
            Ok(Self {
                span: Span {
                    start: name.span.start,
                    end: close_bracket.span.end,
                },
                name,
                index: Some(Box::new(index)),
            })
        } else {
            Ok(Self {
                span: name.span,
                name,
                index: None,
            })
        }
    }
}

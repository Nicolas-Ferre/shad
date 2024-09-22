use crate::atom::parse_token;
use crate::token::{Token, TokenType};
use crate::{AstIdent, AstLiteral};
use logos::Lexer;
use shad_error::{Span, SyntaxError};

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpr {
    /// A unary operation.
    UnaryOperation(AstUnaryOperation),
    /// A binary operation.
    BinaryOperation(AstBinaryOperation),
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
            Self::UnaryOperation(expr) => expr.span,
            Self::BinaryOperation(expr) => expr.span,
            Self::Literal(expr) => expr.span,
            Self::Ident(expr) => expr.span,
            Self::FnCall(expr) => expr.span,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
        let mut expressions = vec![Self::parse_part(lexer)?];
        let mut operators = vec![];
        loop {
            let token = Token::next(&mut lexer.clone())?;
            operators.push((
                match token.type_ {
                    TokenType::Plus => AstBinaryOperator::Add,
                    TokenType::Minus => AstBinaryOperator::Sub,
                    TokenType::Star => AstBinaryOperator::Mul,
                    TokenType::Slash => AstBinaryOperator::Div,
                    TokenType::Percent => AstBinaryOperator::Mod,
                    TokenType::Eq => AstBinaryOperator::Eq,
                    TokenType::NotEq => AstBinaryOperator::NotEq,
                    TokenType::GreaterThanOrEq => AstBinaryOperator::GreaterThanOrEq,
                    TokenType::LessThanOrEq => AstBinaryOperator::LessThanOrEq,
                    TokenType::OpenAngleBracket => AstBinaryOperator::LessThan,
                    TokenType::CloseAngleBracket => AstBinaryOperator::GreaterThan,
                    TokenType::And => AstBinaryOperator::And,
                    TokenType::Or => AstBinaryOperator::Or,
                    _ => break,
                },
                token.span,
            ));
            let _operator = Token::next(lexer)?;
            expressions.push(Self::parse_part(lexer)?);
        }
        if expressions.len() == 1 {
            Ok(expressions.remove(0))
        } else {
            AstBinaryOperation::parse(&expressions, &operators).map(AstExpr::BinaryOperation)
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse_part(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, SyntaxError> {
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
                    Ok(Self::FnCall(AstFnCall::parse(lexer)?))
                } else {
                    Ok(Self::Ident(AstIdent::parse(lexer)?))
                }
            }
            TokenType::Minus => Ok(Self::UnaryOperation(AstUnaryOperation::parse(
                lexer,
                AstUnaryOperator::Neg,
            )?)),
            TokenType::Not => Ok(Self::UnaryOperation(AstUnaryOperation::parse(
                lexer,
                AstUnaryOperator::Not,
            )?)),
            _ => Err(SyntaxError::new(token.span.start, "expected expression")),
        }
    }
}

/// A parsed unary operation.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a unary operation:
/// - `-42`
/// - `-square(value)`
/// - `!boolean_value`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstUnaryOperation {
    /// The span of the unary operation.
    pub span: Span,
    /// The span of the operator.
    pub operator_span: Span,
    /// The unary operator.
    pub operator: AstUnaryOperator,
    /// The operand.
    pub expr: Box<AstExpr>,
}

impl AstUnaryOperation {
    fn parse(
        lexer: &mut Lexer<'_, TokenType>,
        operator: AstUnaryOperator,
    ) -> Result<Self, SyntaxError> {
        let operator_token = Token::next(lexer)?;
        let expr = AstExpr::parse_part(lexer)?;
        Ok(Self {
            span: Span::new(operator_token.span.start, expr.span().end),
            operator_span: operator_token.span,
            operator,
            expr: Box::new(expr),
        })
    }
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstUnaryOperator {
    /// `-` operator.
    Neg,
    /// `!` operator.
    Not,
}

/// A parsed binary operation.
///
/// In case no parenthesis is used to enforce an operator priority, the following priority order is
/// applied:
/// - `*`, `/`, `*`
/// - `+`, `-`
/// - `>`, `<`, `>=`, `<=`, `==`, `!=`
/// - `&&`
/// - `||`
///
/// # Examples
///
/// The following Shad expressions will be parsed as a binary operation:
/// - `2 + 3`
/// - `2 + 3 * 5 - 12` (in this example, `+` is taken as the highest binary operator)
/// - `(2 + 3) * (5 - 12)` (in this example, `*` is taken as the highest binary operator)
/// - `1 == value`
/// - `2 < square(value)`
/// - `a == 2 && b < 5`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstBinaryOperation {
    /// The span of the binary operation.
    pub span: Span,
    /// The span of the operator.
    pub operator_span: Span,
    /// The binary operator.
    pub operator: AstBinaryOperator,
    /// The left operand.
    pub left: Box<AstExpr>,
    /// The right operand.
    pub right: Box<AstExpr>,
}

impl AstBinaryOperation {
    fn parse(
        expressions: &[AstExpr],
        operators: &[(AstBinaryOperator, Span)],
    ) -> Result<Self, SyntaxError> {
        let operator_index = Self::operator_priority()
            .iter()
            .rev()
            .find_map(|ops| operators.iter().position(|op| ops.contains(&op.0)))
            .expect("internal error: expected binary operator");
        let left = if operator_index == 0 {
            Box::new(expressions[0].clone())
        } else {
            Box::new(AstExpr::BinaryOperation(Self::parse(
                &expressions[..=operator_index],
                &operators[..operator_index],
            )?))
        };
        let right = if operator_index == operators.len() - 1 {
            Box::new(expressions[expressions.len() - 1].clone())
        } else {
            Box::new(AstExpr::BinaryOperation(Self::parse(
                &expressions[operator_index + 1..],
                &operators[operator_index + 1..],
            )?))
        };
        Ok(Self {
            span: Span::new(left.span().start, right.span().end),
            operator_span: operators[operator_index].1,
            operator: operators[operator_index].0,
            left,
            right,
        })
    }

    fn operator_priority() -> [Vec<AstBinaryOperator>; 5] {
        [
            vec![
                AstBinaryOperator::Mul,
                AstBinaryOperator::Div,
                AstBinaryOperator::Mod,
            ],
            vec![AstBinaryOperator::Add, AstBinaryOperator::Sub],
            vec![
                AstBinaryOperator::GreaterThan,
                AstBinaryOperator::LessThan,
                AstBinaryOperator::GreaterThanOrEq,
                AstBinaryOperator::LessThanOrEq,
                AstBinaryOperator::Eq,
                AstBinaryOperator::NotEq,
            ],
            vec![AstBinaryOperator::And],
            vec![AstBinaryOperator::Or],
        ]
    }
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstBinaryOperator {
    /// `+` operator.
    Add,
    /// `-` operator.
    Sub,
    /// `*` operator.
    Mul,
    /// `/` operator.
    Div,
    /// `%` operator.
    Mod,
    /// `==` operator.
    Eq,
    /// `!=` operator.
    NotEq,
    /// `>` operator.
    GreaterThan,
    /// `<` operator.
    LessThan,
    /// `>=` operator.
    GreaterThanOrEq,
    /// `<=` operator.
    LessThanOrEq,
    /// `&&` operator.
    And,
    /// `||` operator.
    Or,
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

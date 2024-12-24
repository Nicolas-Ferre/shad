use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstExpr, AstIdent};
use shad_error::{Span, SyntaxError};

/// The function name corresponding to unary `-` operator behavior.
pub const NEG_FN: &str = "__neg__";
/// The function name corresponding to unary `!` operator behavior.
pub const NOT_FN: &str = "__not__";
/// The function name corresponding to binary `+` operator behavior.
pub const ADD_FN: &str = "__add__";
/// The function name corresponding to binary `-` operator behavior.
pub const SUB_FN: &str = "__sub__";
/// The function name corresponding to binary `*` operator behavior.
pub const MUL_FN: &str = "__mul__";
/// The function name corresponding to binary `/` operator behavior.
pub const DIV_FN: &str = "__div__";
/// The function name corresponding to binary `%` operator behavior.
pub const MOD_FN: &str = "__mod__";
/// The function name corresponding to binary `==` operator behavior.
pub const EQ_FN: &str = "__eq__";
/// The function name corresponding to binary `!=` operator behavior.
pub const NE_FN: &str = "__ne__";
/// The function name corresponding to binary `>` operator behavior.
pub const GT_FN: &str = "__gt__";
/// The function name corresponding to binary `<` operator behavior.
pub const LT_FN: &str = "__lt__";
/// The function name corresponding to binary `>=` operator behavior.
pub const GE_FN: &str = "__ge__";
/// The function name corresponding to binary `<=` operator behavior.
pub const LE_FN: &str = "__le__";
/// The function name corresponding to binary `&&` operator behavior.
pub const AND_FN: &str = "__and__";
/// The function name corresponding to binary `||` operator behavior.
pub const OR_FN: &str = "__or__";

/// A parsed function call.
///
/// This corresponds to the identifier of the function, followed between parentheses by
/// comma-separated arguments.
///
/// For binary operations, in case no parenthesis is used to enforce an operator priority,
/// the following priority order is applied:
/// - `*`, `/`, `*`
/// - `+`, `-`
/// - `>`, `<`, `>=`, `<=`, `==`, `!=`
/// - `&&`
/// - `||`
///
/// Following binary operators are supported: `-` and `!`.
///
/// # Examples
///
/// The following Shad examples will be parsed as a function call:
/// - `myfunc()`
/// - `myfunc(expr)`
/// - `myfunc(expr1, expr2)`
/// - `myfunc(expr1, expr2,)`
/// - binary operations like `2 + 3` (call to `__add__` function)
/// - unary operations like `-2` (call to `__neg__` function)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnCall {
    /// The span of the function call.
    pub span: Span,
    /// The function name.
    pub name: AstIdent,
    /// The arguments passed to the function.
    pub args: Vec<AstFnCallArg>,
    /// Whether the function call is done using an operator.
    pub is_operator: bool,
}

impl AstFnCall {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = AstIdent::parse(lexer)?;
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut args = vec![];
        while parse_token(&mut lexer.clone(), TokenType::CloseParenthesis).is_err() {
            args.push(AstFnCallArg::parse(lexer)?);
            if parse_token_option(lexer, TokenType::Comma)?.is_none() {
                break;
            }
        }
        let close_parenthesis = parse_token(lexer, TokenType::CloseParenthesis)?;
        Ok(Self {
            span: Span::join(&name.span, &close_parenthesis.span),
            name,
            args,
            is_operator: false,
        })
    }

    pub(crate) fn parse_binary_operation(
        lexer: &mut Lexer<'_>,
        expressions: &[AstExpr],
        operators: &[(TokenType, Span)],
    ) -> Result<Self, SyntaxError> {
        let operator_index = Self::operator_priority()
            .iter()
            .rev()
            .find_map(|ops| {
                operators
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|(_, op)| ops.contains(&op.0))
                    .map(|(index, _)| index)
                    .next()
            })
            .expect("internal error: expected binary operator");
        let left = if operator_index == 0 {
            expressions[0].clone()
        } else {
            Self::parse_binary_operation(
                lexer,
                &expressions[..=operator_index],
                &operators[..operator_index],
            )?
            .into()
        };
        let right = if operator_index == operators.len() - 1 {
            expressions[expressions.len() - 1].clone()
        } else {
            Self::parse_binary_operation(
                lexer,
                &expressions[operator_index + 1..],
                &operators[operator_index + 1..],
            )?
            .into()
        };
        Ok(Self {
            span: Span::join(&left.span, &right.span),
            name: AstIdent {
                span: operators[operator_index].1.clone(),
                label: Self::binary_operator_fn_name(operators[operator_index].0).into(),
                id: lexer.next_id(),
            },
            args: vec![left.into(), right.into()],
            is_operator: true,
        })
    }

    pub(crate) fn parse_unary_operation(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let operator_token = lexer.next_token()?;
        let expr = AstExpr::parse(lexer)?;
        Ok(Self {
            span: Span::join(&operator_token.span, &expr.span),
            name: AstIdent {
                span: operator_token.span,
                label: Self::unary_operator_fn_name(operator_token.type_).into(),
                id: lexer.next_id(),
            },
            args: vec![expr.into()],
            is_operator: true,
        })
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn binary_operator_fn_name(operator: TokenType) -> &'static str {
        match operator {
            TokenType::Plus => ADD_FN,
            TokenType::Minus => SUB_FN,
            TokenType::Star => MUL_FN,
            TokenType::Slash => DIV_FN,
            TokenType::Percent => MOD_FN,
            TokenType::Eq => EQ_FN,
            TokenType::NotEq => NE_FN,
            TokenType::CloseAngleBracket => GT_FN,
            TokenType::OpenAngleBracket => LT_FN,
            TokenType::GreaterThanOrEq => GE_FN,
            TokenType::LessThanOrEq => LE_FN,
            TokenType::And => AND_FN,
            TokenType::Or => OR_FN,
            _ => unreachable!("internal error: unsupported binary operator"),
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn unary_operator_fn_name(operator: TokenType) -> &'static str {
        match operator {
            TokenType::Minus => NEG_FN,
            TokenType::Not => NOT_FN,
            _ => unreachable!("internal error: unsupported unary operator"),
        }
    }

    fn operator_priority() -> [Vec<TokenType>; 5] {
        [
            vec![TokenType::Star, TokenType::Slash, TokenType::Percent],
            vec![TokenType::Plus, TokenType::Minus],
            vec![
                TokenType::CloseAngleBracket,
                TokenType::OpenAngleBracket,
                TokenType::GreaterThanOrEq,
                TokenType::LessThanOrEq,
                TokenType::Eq,
                TokenType::NotEq,
            ],
            vec![TokenType::And],
            vec![TokenType::Or],
        ]
    }
}

/// A parsed argument passed to a function call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFnCallArg {
    /// The parameter name when explicitly specified.
    pub name: Option<AstIdent>,
    /// The argument value.
    pub value: AstExpr,
}

impl AstFnCallArg {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let name = Self::parse_name(&mut lexer.clone()).and_then(|_| Self::parse_name(lexer));
        let value = AstExpr::parse(lexer)?;
        Ok(Self { name, value })
    }

    fn parse_name(lexer: &mut Lexer<'_>) -> Option<AstIdent> {
        let name = AstIdent::parse(lexer).ok()?;
        parse_token(lexer, TokenType::Colon).ok()?;
        Some(name)
    }
}

impl From<AstExpr> for AstFnCallArg {
    fn from(value: AstExpr) -> Self {
        Self { name: None, value }
    }
}

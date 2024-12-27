use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{item, AstStatement};
use shad_error::SyntaxError;
use std::str::FromStr;

/// A parsed run block.
///
/// # Examples
///
/// The following Shad examples will be parsed as a run block:
/// - `run { my_buffer = 2.; }`
/// - `run priority 10 { my_buffer = 2.; }`
/// - `run priority -42 { my_buffer = 2.; }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstRunItem {
    /// The statements inside the block.
    pub statements: Vec<AstStatement>,
    /// The execution priority.
    pub priority: Option<i32>,
    /// The unique ID of the `run` block.
    pub id: u64,
}

impl AstRunItem {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        parse_token(lexer, TokenType::Run)?;
        let priority = Self::parse_priority(lexer)?;
        let statements = item::parse_statement_block(lexer)?;
        Ok(Self {
            statements,
            priority,
            id: lexer.next_id(),
        })
    }

    fn parse_priority(lexer: &mut Lexer<'_>) -> Result<Option<i32>, SyntaxError> {
        Ok(
            if parse_token_option(lexer, TokenType::Priority)?.is_some() {
                Some(Self::parse_priority_value(lexer)?)
            } else {
                None
            },
        )
    }

    fn parse_priority_value(lexer: &mut Lexer<'_>) -> Result<i32, SyntaxError> {
        let is_neg = parse_token_option(lexer, TokenType::Minus)?.is_some();
        let value = parse_token(lexer, TokenType::I32Literal)?;
        let value_with_operator = if is_neg {
            format!("-{}", value.slice)
        } else {
            value.slice.into()
        };
        i32::from_str(&value_with_operator.replace('_', "")).map_err(|_| {
            SyntaxError::new(
                value.span.start,
                lexer.module(),
                "`i32` literal out of range".to_string(),
            )
        })
    }
}

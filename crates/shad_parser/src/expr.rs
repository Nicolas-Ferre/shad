use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, Token, TokenType};
use crate::{AstFnCall, AstIdent, AstLiteral};
use shad_error::{Span, SyntaxError};
use std::mem;

/// A parsed expression.
///
/// # Examples
///
/// The following Shad examples will be parsed as an expression:
/// - `my_var`
/// - `my_var.field`
/// - `my_var.field.subfield.subsubfield`
/// - `my_func(42)`
/// - `my_func(42).field`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExpr {
    /// The span of the value.
    pub span: Span,
    /// The value root.
    pub root: AstExprRoot,
    /// The fields referred after the root part.
    pub fields: Vec<AstIdent>,
}

const LITERALS: [TokenType; 5] = [
    TokenType::F32Literal,
    TokenType::U32Literal,
    TokenType::I32Literal,
    TokenType::True,
    TokenType::False,
];

impl AstExpr {
    /// Replaces the value root part by another value.
    pub fn replace_root(&mut self, new_root: Self) {
        self.root = new_root.root;
        self.fields = [new_root.fields, mem::take(&mut self.fields)].concat();
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut expressions = vec![Self::parse_part(lexer)?];
        let mut operators = vec![];
        loop {
            let token = Token::next(&mut lexer.clone())?;
            if [
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Star,
                TokenType::Slash,
                TokenType::Percent,
                TokenType::Eq,
                TokenType::NotEq,
                TokenType::GreaterThanOrEq,
                TokenType::LessThanOrEq,
                TokenType::OpenAngleBracket,
                TokenType::CloseAngleBracket,
                TokenType::And,
                TokenType::Or,
            ]
            .contains(&token.type_)
            {
                operators.push((token.type_, token.span));
            } else {
                break;
            }
            let _operator = Token::next(lexer)?;
            expressions.push(Self::parse_part(lexer)?);
        }
        if expressions.len() == 1 {
            Ok(expressions.remove(0))
        } else {
            AstFnCall::parse_binary_operation(lexer, &expressions, &operators).map(AstFnCall::into)
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse_part(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        match Token::next(&mut lexer.clone())?.type_ {
            TokenType::OpenParenthesis => {
                parse_token(lexer, TokenType::OpenParenthesis)?;
                let expr = Self::parse(lexer)?;
                parse_token(lexer, TokenType::CloseParenthesis)?;
                Ok(expr)
            }
            TokenType::Minus | TokenType::Not => {
                Ok(AstFnCall::parse_unary_operation(lexer)?.into())
            }
            TokenType::F32Literal
            | TokenType::U32Literal
            | TokenType::I32Literal
            | TokenType::True
            | TokenType::False
            | TokenType::Ident => {
                let mut tmp_lexer = lexer.clone();
                let root = if LITERALS.contains(&Token::next(&mut lexer.clone())?.type_) {
                    AstExprRoot::Literal(AstLiteral::parse(lexer)?)
                } else if AstIdent::parse(&mut tmp_lexer).is_ok()
                    && parse_token(&mut tmp_lexer, TokenType::OpenParenthesis).is_ok()
                {
                    AstExprRoot::FnCall(AstFnCall::parse(lexer)?)
                } else {
                    AstExprRoot::Ident(AstIdent::parse(lexer)?)
                };
                let mut fields = vec![];
                while parse_token_option(lexer, TokenType::Dot)?.is_some() {
                    fields.push(AstIdent::parse(lexer)?);
                }
                Ok(Self {
                    span: Span::join(
                        root.span(),
                        fields.last().map_or(root.span(), |field| &field.span),
                    ),
                    root,
                    fields,
                })
            }
            _ => Err(SyntaxError::new(
                Token::next(&mut lexer.clone())?.span.start,
                lexer.module.clone(),
                "expected expression",
            )),
        }
    }
}

impl From<AstIdent> for AstExpr {
    fn from(ident: AstIdent) -> Self {
        Self {
            span: ident.span.clone(),
            root: AstExprRoot::Ident(ident),
            fields: vec![],
        }
    }
}

impl From<AstFnCall> for AstExpr {
    fn from(call: AstFnCall) -> Self {
        Self {
            span: call.span.clone(),
            root: AstExprRoot::FnCall(call),
            fields: vec![],
        }
    }
}

impl From<AstLiteral> for AstExpr {
    fn from(literal: AstLiteral) -> Self {
        Self {
            span: literal.span.clone(),
            root: AstExprRoot::Literal(literal),
            fields: vec![],
        }
    }
}

/// A parsed value root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExprRoot {
    /// An identifier.
    Ident(AstIdent),
    /// A function call.
    FnCall(AstFnCall),
    /// A literal.
    Literal(AstLiteral),
}

impl AstExprRoot {
    /// Returns the span of the value.
    pub fn span(&self) -> &Span {
        match self {
            Self::Ident(value) => &value.span,
            Self::FnCall(value) => &value.span,
            Self::Literal(value) => &value.span,
        }
    }
}

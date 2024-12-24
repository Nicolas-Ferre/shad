use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
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
const BINARY_OPERATORS: [TokenType; 13] = [
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
];

impl AstExpr {
    /// Replaces the value root part by another value.
    pub fn replace_root(&mut self, new_root: Self) {
        self.root = new_root.root;
        self.fields = [new_root.fields, mem::take(&mut self.fields)].concat();
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut expressions = vec![Self::parse_operand(lexer)?];
        let mut operators = vec![];
        loop {
            let token = lexer.clone().next_token()?;
            if BINARY_OPERATORS.contains(&token.type_) {
                operators.push((token.type_, token.span));
            } else {
                break;
            }
            let _operator = lexer.next_token()?;
            expressions.push(Self::parse_operand(lexer)?);
        }
        if expressions.len() == 1 {
            Ok(expressions.remove(0))
        } else {
            Ok(AstFnCall::parse_binary_operation(lexer, &expressions, &operators)?.into())
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_operand(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        match lexer.clone().next_token()?.type_ {
            TokenType::OpenParenthesis => {
                parse_token(lexer, TokenType::OpenParenthesis)?;
                let mut expr = Self::parse(lexer)?;
                parse_token(lexer, TokenType::CloseParenthesis)?;
                expr.fields.extend(Self::parse_fields(lexer)?);
                expr.span = Self::span(&expr.root, &expr.fields);
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
                let mut expr = Self::parse_operand_start(lexer)?;
                let mut tails = vec![];
                while parse_token_option(lexer, TokenType::Dot)?.is_some() {
                    tails.push((AstFnCall::parse(lexer)?, Self::parse_fields(lexer)?));
                }
                for (mut call, fields) in tails {
                    call.is_first_arg_external = true;
                    call.args.insert(0, expr.clone().into());
                    expr = Self {
                        span: Span::join(&expr.span, &call.span),
                        root: AstExprRoot::FnCall(call),
                        fields,
                    };
                }
                Ok(expr)
            }
            _ => Err(SyntaxError::new(
                lexer.clone().next_token()?.span.start,
                lexer.module(),
                "expected expression",
            )),
        }
    }

    fn parse_operand_start(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut tmp_lexer = lexer.clone();
        let root = if LITERALS.contains(&lexer.clone().next_token()?.type_) {
            AstExprRoot::Literal(AstLiteral::parse(lexer)?)
        } else if AstIdent::parse(&mut tmp_lexer).is_ok()
            && parse_token(&mut tmp_lexer, TokenType::OpenParenthesis).is_ok()
        {
            AstExprRoot::FnCall(AstFnCall::parse(lexer)?)
        } else {
            AstExprRoot::Ident(AstIdent::parse(lexer)?)
        };
        let fields = Self::parse_fields(lexer)?;
        Ok(Self {
            span: Self::span(&root, &fields),
            root,
            fields,
        })
    }

    fn span(root: &AstExprRoot, fields: &[AstIdent]) -> Span {
        Span::join(
            root.span(),
            fields.last().map_or(root.span(), |field| &field.span),
        )
    }

    fn parse_fields(lexer: &mut Lexer<'_>) -> Result<Vec<AstIdent>, SyntaxError> {
        let mut fields = vec![];
        loop {
            let mut tmp_lexer = lexer.clone();
            if parse_token(&mut tmp_lexer, TokenType::Dot).is_err() {
                break;
            }
            AstIdent::parse(&mut tmp_lexer)?;
            if tmp_lexer.next_token()?.type_ == TokenType::OpenParenthesis {
                break;
            }
            parse_token(lexer, TokenType::Dot)?;
            fields.push(AstIdent::parse(lexer)?);
        }
        Ok(fields)
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

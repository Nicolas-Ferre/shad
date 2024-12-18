use crate::atom::{parse_token, parse_token_option};
use crate::token::{Lexer, TokenType};
use crate::{AstFnCall, AstIdent};
use shad_error::{Span, SyntaxError};
use std::mem;

/// A parsed value.
///
/// # Examples
///
/// The following Shad expressions will be parsed as a value:
/// - `my_var`
/// - `my_var.field`
/// - `my_var.field.subfield.subsubfield`
/// - `my_func(42)`
/// - `my_func(42).field`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstValue {
    /// The span of the value.
    pub span: Span,
    /// The value root.
    pub root: AstValueRoot,
    /// The fields referred after the root part.
    pub fields: Vec<AstIdent>,
}

impl AstValue {
    /// Replaces the value root part by another value.
    pub fn replace_root(&mut self, new_root: Self) {
        self.root = new_root.root;
        self.fields = [new_root.fields, mem::take(&mut self.fields)].concat();
    }

    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let mut tmp_lexer = lexer.clone();
        let is_fn_call = AstIdent::parse(&mut tmp_lexer).is_ok()
            && parse_token(&mut tmp_lexer, TokenType::OpenParenthesis).is_ok();
        let root = if is_fn_call {
            AstValueRoot::FnCall(AstFnCall::parse(lexer)?)
        } else {
            AstValueRoot::Ident(AstIdent::parse(lexer)?)
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
}

impl From<AstIdent> for AstValue {
    fn from(ident: AstIdent) -> Self {
        Self {
            span: ident.span.clone(),
            root: AstValueRoot::Ident(ident),
            fields: vec![],
        }
    }
}

impl From<AstFnCall> for AstValue {
    fn from(call: AstFnCall) -> Self {
        Self {
            span: call.span.clone(),
            root: AstValueRoot::FnCall(call),
            fields: vec![],
        }
    }
}

/// A parsed value root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstValueRoot {
    /// An identifier.
    Ident(AstIdent),
    /// A function call.
    FnCall(AstFnCall),
}

impl AstValueRoot {
    /// Returns the span of the value.
    pub fn span(&self) -> &Span {
        match self {
            Self::Ident(value) => &value.span,
            Self::FnCall(value) => &value.span,
        }
    }
}

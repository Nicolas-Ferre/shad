use crate::error::SyntaxError;
use logos::{Lexer, Logos};
use std::fmt::Debug;

/// The span of a group of token in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The byte offset of the span start.
    pub start: usize,
    /// The byte offset of the span end.
    pub end: usize,
}

impl Span {
    /// Creates a span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) fn from_logos(span: logos::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum TokenType {
    #[token("buf")]
    Buf,

    #[token("=")]
    Equal,

    #[token(";")]
    SemiColon,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("[0-9][0-9_]*\\.([0-9][0-9_]*)?")]
    F32Literal,
}

impl TokenType {
    // coverage: off (not all labels are used in practice)
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Buf => "`buf`",
            Self::Equal => "`=`",
            Self::SemiColon => "`;`",
            Self::Ident => "identifier",
            Self::F32Literal => "`f32` literal",
        }
    }
    // coverage: on
}

#[derive(Debug)]
pub(crate) struct Token<'a> {
    pub(crate) type_: TokenType,
    pub(crate) span: logos::Span,
    pub(crate) slice: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) fn next(lexer: &mut Lexer<'a, TokenType>) -> Result<Self, SyntaxError> {
        Ok(Self {
            type_: lexer
                .next()
                .ok_or_else(|| SyntaxError::new(lexer.span().start, "unexpected end of file"))?
                .map_err(|()| SyntaxError::new(lexer.span().start, "unexpected token"))?,
            span: lexer.span(),
            slice: lexer.slice(),
        })
    }
}

use logos::{Lexer, Logos};
use shad_error::SyntaxError;
use std::fmt::Debug;

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

    #[regex("[0-9][0-9_]*u")]
    U32Literal,

    #[regex("[0-9][0-9_]*")]
    I32Literal,
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
            Self::U32Literal => "`u32` literal",
            Self::I32Literal => "`i32` literal",
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

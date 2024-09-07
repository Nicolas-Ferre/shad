use annotate_snippets::{Level, Renderer, Snippet};
use logos::{Lexer, Logos};
use std::fmt::{Debug, Display, Formatter};
use std::{error, io};

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    Parsing(ParsingError),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parsing(err) => Display::fmt(err, f),
            Error::Io(err) => Display::fmt(err, f),
        }
    }
}

impl error::Error for Error {}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsingError {
    pub offset: usize,
    pub message: String,
    pub pretty_message: String,
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}

impl error::Error for ParsingError {}

impl ParsingError {
    pub(crate) fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
            pretty_message: String::new(),
        }
    }

    #[allow(clippy::range_plus_one)]
    pub(crate) fn with_pretty_message(self, file_path: &str, code: &str) -> Self {
        let message = Level::Error.title(&self.message).snippet(
            Snippet::source(code)
                .fold(true)
                .origin(file_path)
                .annotation(
                    Level::Error
                        .span(self.offset.min(code.len() - 1)..(self.offset + 1).min(code.len()))
                        .label("here"),
                ),
        );
        let pretty_message = format!("{}", Renderer::styled().render(message));
        Self {
            offset: self.offset,
            message: self.message,
            pretty_message,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn new(span: logos::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum TokenType {
    #[token("let")]
    Let,

    #[token("return")]
    Return,

    #[token("for")]
    For,

    #[token("loop")]
    Loop,

    #[token("in")]
    In,

    #[token("fn")]
    Fn,

    #[token("cpu")]
    Cpu,

    #[token("gpu")]
    Gpu,

    #[token("->")]
    Arrow,

    #[token("+")]
    Add,

    #[token("-")]
    Sub,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("=")]
    Equal,

    #[token(",")]
    Comma,

    #[token(";")]
    SemiColon,

    #[token(":")]
    Colon,

    #[token(".")]
    Dot,

    #[token("(")]
    OpenParenthesis,

    #[token(")")]
    CloseParenthesis,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token("<")]
    OpenAngleBracket,

    #[token(">")]
    CloseAngleBracket,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("[0-9][0-9_]*\\.([0-9][0-9_]*)?")]
    FloatLiteral,

    #[regex("[0-9][0-9_]*")]
    IntLiteral,
}

impl TokenType {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Let => "`let`",
            Self::Return => "`return`",
            Self::For => "`for`",
            Self::Loop => "`loop`",
            Self::In => "`in`",
            Self::Fn => "`fn`",
            Self::Cpu => "`cpu`",
            Self::Gpu => "`gpu`",
            Self::Arrow => "`->`",
            Self::Add => "`+`",
            Self::Sub => "`-`",
            Self::Mul => "`*`",
            Self::Div => "`/`",
            Self::Equal => "`=`",
            Self::Comma => "`,`",
            Self::SemiColon => "`:`",
            Self::Colon => "`;`",
            Self::Dot => "`.`",
            Self::OpenParenthesis => "`(`",
            Self::CloseParenthesis => "`)`",
            Self::OpenBrace => "`{`",
            Self::CloseBrace => "`}`",
            Self::OpenSquareBracket => "`[`",
            Self::CloseSquareBracket => "`]`",
            Self::OpenAngleBracket => "`<`",
            Self::CloseAngleBracket => "`>`",
            Self::Ident => "identifier",
            Self::FloatLiteral => "float literal",
            Self::IntLiteral => "int literal",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Token<'a> {
    pub(crate) type_: TokenType,
    pub(crate) span: logos::Span,
    pub(crate) slice: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) fn next(lexer: &mut Lexer<'a, TokenType>) -> Result<Self, ParsingError> {
        Ok(Self {
            type_: lexer
                .next()
                .ok_or_else(|| ParsingError::new(lexer.span().start, "unexpected end of file"))?
                .map_err(|()| ParsingError::new(lexer.span().start, "unexpected token"))?,
            span: lexer.span(),
            slice: lexer.slice(),
        })
    }
}

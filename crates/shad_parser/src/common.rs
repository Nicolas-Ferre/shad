use annotate_snippets::{Level, Renderer, Snippet};
use logos::{Lexer, Logos};
use std::fmt::{Debug, Display, Formatter};
use std::{error, io};

/// An error obtained when trying to parse a Shad code.
#[derive(Debug)]
pub enum Error {
    /// A parsing error.
    Syntax(SyntaxError),
    /// An I/O error.
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Syntax(err1), Self::Syntax(err2)) => err1 == err2,
            (Self::Io(err1), Self::Io(err2)) => err1.to_string() == err2.to_string(),
            _ => false,
        }
    }
}

impl Eq for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(err) => Display::fmt(err, f),
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl error::Error for Error {}

/// A syntax error obtained when trying to parse a Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// The byte offset where the error is located in the file.
    pub offset: usize,
    /// The error message.
    pub message: String,
    /// The formatted error message.
    pub pretty_message: String,
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_message)
    }
}

impl error::Error for SyntaxError {}

impl SyntaxError {
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
    FloatLiteral,
}

impl TokenType {
    // coverage: off (not all labels are used in practice)
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Buf => "`buf`",
            Self::Equal => "`=`",
            Self::SemiColon => "`;`",
            Self::Ident => "identifier",
            Self::FloatLiteral => "float literal",
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

use logos::Logos;
use shad_error::{ModuleLocation, Span, SyntaxError};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum TokenType {
    #[token("import")]
    Import,

    #[token("buf")]
    Buf,

    #[token("run")]
    Run,

    #[token("fn")]
    Fn,

    #[token("gpu")]
    Gpu,

    #[token("pub")]
    Pub,

    #[token("var")]
    Var,

    #[token("ref")]
    Ref,

    #[token("return")]
    Return,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("!")]
    Not,

    #[token("==")]
    Eq,

    #[token("!=")]
    NotEq,

    #[token(">=")]
    GreaterThanOrEq,

    #[token("<=")]
    LessThanOrEq,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("=")]
    Assigment,

    #[token(",")]
    Comma,

    #[token(";")]
    SemiColon,

    #[token(":")]
    Colon,

    #[token(".")]
    Dot,

    #[token("->")]
    Arrow,

    #[token("(")]
    OpenParenthesis,

    #[token(")")]
    CloseParenthesis,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("<")]
    OpenAngleBracket,

    #[token(">")]
    CloseAngleBracket,

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
            Self::Import => "`import`",
            Self::Buf => "`buf`",
            Self::Run => "`run`",
            Self::Fn => "`fn`",
            Self::Gpu => "`gpu`",
            Self::Pub => "`pub`",
            Self::Var => "`var`",
            Self::Ref => "`ref`",
            Self::Return => "`return`",
            Self::True => "`true`",
            Self::False => "`false`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Percent => "`%`",
            Self::Not => "`!`",
            Self::Eq => "`==`",
            Self::NotEq => "`!=`",
            Self::GreaterThanOrEq => "`>=`",
            Self::LessThanOrEq => "`<=`",
            Self::And => "`&&`",
            Self::Or => "`||`",
            Self::Assigment => "`=`",
            Self::Comma => "`,`",
            Self::SemiColon => "`;`",
            Self::Colon => "`:`",
            Self::Dot => "`.`",
            Self::Arrow => "`->`",
            Self::OpenParenthesis => "`(`",
            Self::CloseParenthesis => "`)`",
            Self::OpenBrace => "`{`",
            Self::CloseBrace => "`}`",
            Self::OpenAngleBracket => "`<`",
            Self::CloseAngleBracket => "`>`",
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
    pub(crate) span: Span,
    pub(crate) slice: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) fn next(lexer: &mut Lexer<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            type_: lexer
                .inner
                .next()
                .ok_or_else(|| {
                    SyntaxError::new(lexer.inner.span().start, "unexpected end of file")
                })?
                .map_err(|()| SyntaxError::new(lexer.inner.span().start, "unexpected token"))?,
            span: Span::new(
                lexer.inner.span().start,
                lexer.inner.span().end,
                lexer.module.clone(),
            ),
            slice: lexer.inner.slice(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Lexer<'a> {
    pub(crate) inner: logos::Lexer<'a, TokenType>,
    pub(crate) module: Rc<ModuleLocation>,
    pub(crate) next_id: u64,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(code: &'a str, path: &str, module: &str) -> Self {
        Self {
            inner: TokenType::lexer(code),
            module: Rc::new(ModuleLocation {
                name: module.into(),
                path: path.into(),
                code: code.into(),
            }),
            next_id: 1000,
        }
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

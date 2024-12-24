use logos::Logos;
use shad_error::{ModuleLocation, Span, SyntaxError};
use std::fmt::Debug;
use std::mem;
use std::rc::Rc;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum TokenType {
    #[token("import")]
    Import,

    #[token("struct")]
    Struct,

    #[token("buf")]
    Buf,

    #[token("run")]
    Run,

    #[token("priority")]
    Priority,

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
            Self::Struct => "`struct`",
            Self::Buf => "`buf`",
            Self::Run => "`run`",
            Self::Priority => "`priority`",
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

#[derive(Debug, Clone)]
pub(crate) struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenType>,
    next_token: NextToken,
    module: Rc<ModuleLocation>,
    next_id: u64,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(
        cleaned_code: &'a str,
        raw_code: &'a str,
        path: &str,
        module: &str,
        next_id: u64,
    ) -> Self {
        Self {
            inner: TokenType::lexer(cleaned_code),
            module: Rc::new(ModuleLocation {
                name: module.into(),
                path: path.into(),
                code: raw_code.into(),
            }),
            next_id,
            next_token: NextToken::Actual,
        }
    }

    pub(crate) fn module(&self) -> Rc<ModuleLocation> {
        self.module.clone()
    }

    pub(crate) fn has_next_token(&self) -> bool {
        self.inner.clone().next().is_some()
    }

    pub(crate) fn next_token(&mut self) -> Result<Token<'a>, SyntaxError> {
        match mem::replace(&mut self.next_token, NextToken::Actual) {
            NextToken::Actual => {
                let token = self.next_token_type()?;
                if token == TokenType::F32Literal
                    && self.inner.slice().ends_with('.')
                    && self.clone().next_token()?.type_ == TokenType::Ident
                {
                    Ok(self.first_split_f32_literal_part())
                } else {
                    Ok(Token {
                        type_: token,
                        span: Span::new(
                            self.inner.span().start,
                            self.inner.span().end,
                            self.module(),
                        ),
                        slice: self.inner.slice(),
                    })
                }
            }
            NextToken::Dot(span) => Ok(Token {
                type_: TokenType::Dot,
                span,
                slice: ".",
            }),
        }
    }

    fn first_split_f32_literal_part(&mut self) -> Token<'a> {
        self.next_token = NextToken::Dot(Span::new(
            self.inner.span().end - 2,
            self.inner.span().end,
            self.module(),
        ));
        Token {
            type_: TokenType::I32Literal,
            span: Span::new(
                self.inner.span().start,
                self.inner.span().end - 1,
                self.module(),
            ),
            slice: &self.inner.slice()[..self.inner.slice().len() - 1],
        }
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn next_token_type(&mut self) -> Result<TokenType, SyntaxError> {
        self.inner
            .next()
            .ok_or_else(|| {
                SyntaxError::new(
                    self.inner.span().start,
                    self.module(),
                    "unexpected end of file",
                )
            })?
            .map_err(|()| {
                SyntaxError::new(self.inner.span().start, self.module(), "unexpected token")
            })
    }
}

#[derive(Debug)]
pub(crate) struct Token<'a> {
    pub(crate) type_: TokenType,
    pub(crate) span: Span,
    pub(crate) slice: &'a str,
}

#[derive(Debug, Clone)]
enum NextToken {
    Actual,
    Dot(Span),
}

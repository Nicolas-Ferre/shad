use crate::atoms::parse_token;
use crate::common::{Token, TokenType};
use crate::{Ident, ParsingError, Span, Statement};
use logos::Lexer;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnItem {
    pub span: Span,
    pub name: Ident,
    pub params: Vec<FnParam>,
    pub return_type: Option<Type>,
    pub statements: Vec<Statement>,
    pub qualifier: FnQualifier,
}

impl FnItem {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        match Token::next(&mut lexer.clone())?.type_ {
            token @ (TokenType::Cpu | TokenType::Gpu) => Self::parse_extern_fn(lexer, token),
            _ => Self::parse_standard_fn(lexer),
        }
    }

    fn parse_standard_fn(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let fn_ = parse_token(lexer, TokenType::Fn)?;
        let name = Ident::parse(lexer)?;
        let params = Self::parse_params(lexer)?;
        let return_type = Self::parse_return_type(lexer)?;
        parse_token(lexer, TokenType::OpenBrace)?;
        let statements = Statement::parse_many(lexer)?;
        let close_brace = parse_token(lexer, TokenType::CloseBrace)?;
        Ok(Self {
            span: Span {
                start: fn_.span.start,
                end: close_brace.span.end,
            },
            name,
            params,
            return_type,
            statements,
            qualifier: FnQualifier::None,
        })
    }

    fn parse_extern_fn(
        lexer: &mut Lexer<'_, TokenType>,
        qualifier_token: TokenType,
    ) -> Result<Self, ParsingError> {
        let qualifier = parse_token(lexer, qualifier_token)?;
        parse_token(lexer, TokenType::Fn)?;
        let name = Ident::parse(lexer)?;
        let params = Self::parse_params(lexer)?;
        let return_type = Self::parse_return_type(lexer)?;
        let semi_colon = parse_token(lexer, TokenType::SemiColon)?;
        Ok(Self {
            span: Span {
                start: qualifier.span.start,
                end: semi_colon.span.end,
            },
            name,
            params,
            return_type,
            statements: vec![],
            qualifier: if qualifier_token == TokenType::Cpu {
                FnQualifier::Cpu
            } else {
                FnQualifier::Gpu
            },
        })
    }

    fn parse_params(lexer: &mut Lexer<'_, TokenType>) -> Result<Vec<FnParam>, ParsingError> {
        parse_token(lexer, TokenType::OpenParenthesis)?;
        let mut params = vec![];
        loop {
            let token = Token::next(&mut lexer.clone())?;
            if token.type_ == TokenType::CloseParenthesis {
                break;
            }
            params.push(FnParam::parse(lexer)?);
            let token = Token::next(&mut lexer.clone())?;
            if token.type_ == TokenType::Comma {
                Token::next(lexer)?;
            } else if token.type_ != TokenType::CloseParenthesis {
                break;
            }
        }
        parse_token(lexer, TokenType::CloseParenthesis)?;
        Ok(params)
    }

    fn parse_return_type(lexer: &mut Lexer<'_, TokenType>) -> Result<Option<Type>, ParsingError> {
        Ok(
            if Token::next(&mut lexer.clone())?.type_ == TokenType::Arrow {
                parse_token(lexer, TokenType::Arrow)?;
                Some(Type::parse(lexer)?)
            } else {
                None
            },
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnParam {
    pub name: Ident,
    pub type_: Type,
}

impl FnParam {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let name = Ident::parse(lexer)?;
        parse_token(lexer, TokenType::Colon)?;
        let type_ = Type::parse(lexer)?;
        Ok(Self { name, type_ })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub name: Ident,
    pub generic: Option<Box<Type>>,
}

impl Type {
    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let name = Ident::parse(lexer)?;
        let generic = if Token::next(&mut lexer.clone())?.type_ == TokenType::OpenAngleBracket {
            parse_token(lexer, TokenType::OpenAngleBracket)?;
            let generic = Self::parse(lexer)?;
            parse_token(lexer, TokenType::CloseAngleBracket)?;
            Some(Box::new(generic))
        } else {
            None
        };
        Ok(Self { name, generic })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnQualifier {
    None,
    Cpu,
    Gpu,
}

use crate::common::{Token, TokenType};
use crate::{Error, FnItem, ParsingError};
use logos::{Lexer, Logos};
use std::{fs, io, iter};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<FnItem>,
}

impl Program {
    pub fn parse_file(path: &str) -> Result<Self, Error> {
        let raw_code = Self::retrieve_code(path).map_err(Error::Io)?;
        let cleaned_code = Self::remove_comments(&raw_code);
        let mut lexer = TokenType::lexer(&cleaned_code);
        Self::parse(&mut lexer)
            .map_err(|e| e.with_pretty_message(path, &raw_code))
            .map_err(Error::Parsing)
    }

    fn parse(lexer: &mut Lexer<'_, TokenType>) -> Result<Self, ParsingError> {
        let mut items = vec![];
        while Token::next(&mut lexer.clone()).is_ok() {
            items.push(FnItem::parse(lexer)?);
        }
        Ok(Self { items })
    }

    fn retrieve_code(path: &str) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn remove_comments(code: &str) -> String {
        code.split('\n')
            .map(|line| {
                line.split_once("//").map_or(line.to_string(), |line| {
                    iter::once(line.0)
                        .chain(iter::repeat(" ").take(line.1.len() + "//".len()))
                        .collect::<String>()
                })
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

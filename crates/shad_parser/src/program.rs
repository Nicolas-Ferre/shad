use crate::common::{Token, TokenType};
use crate::error::{Error, SyntaxError};
use crate::Item;
use logos::{Lexer, Logos};
use std::path::Path;
use std::{fs, io, iter};

/// A parsed Shad program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProgram {
    /// The raw Shad code.
    pub code: String,
    /// The path to the Shad code file.
    pub path: String,
    /// All the items.
    pub items: Vec<Item>,
}

impl ParsedProgram {
    /// Parses a file containing Shad code.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let code = Self::retrieve_code(&path).map_err(Error::Io)?;
        Self::parse_str(&code, path.as_ref().to_str().unwrap_or_default())
    }

    /// Parses a string containing Shad code.
    ///
    /// A `path` can be provided to improve formatted error messages.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn parse_str(code: &str, path: &str) -> Result<Self, Error> {
        let cleaned_code = Self::remove_comments(code);
        let mut lexer = TokenType::lexer(&cleaned_code);
        Self::parse(&mut lexer, code, path)
            .map_err(|e| e.with_pretty_message(path, code))
            .map_err(Error::Syntax)
    }

    fn parse(
        lexer: &mut Lexer<'_, TokenType>,
        code: &str,
        path: &str,
    ) -> Result<Self, SyntaxError> {
        let mut items = vec![];
        while Token::next(&mut lexer.clone()).is_ok() {
            items.push(Item::parse(lexer)?);
        }
        Ok(Self {
            code: code.to_string(),
            path: path.to_string(),
            items,
        })
    }

    fn retrieve_code(path: &impl AsRef<Path>) -> io::Result<String> {
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
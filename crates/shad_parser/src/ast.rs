use crate::token::{Token, TokenType};
use crate::AstItem;
use logos::{Lexer, Logos};
use shad_error::{Error, SyntaxError};
use std::path::Path;
use std::{fs, io, iter};

/// The Abstract Syntax Tree of a parsed Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    /// The raw Shad code.
    pub code: String,
    /// The path to the Shad code file.
    pub path: String,
    /// All the items.
    pub items: Vec<AstItem>,
}

impl Ast {
    /// Parses a file containing Shad code to obtain an AST.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let code = Self::retrieve_code(&path).map_err(Error::Io)?;
        Self::from_str(&code, path.as_ref().to_str().unwrap_or_default())
    }

    /// Parses a string containing Shad code to obtain an AST.
    ///
    /// A `path` can be provided to improve formatted error messages.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn from_str(code: &str, path: &str) -> Result<Self, Error> {
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
            items.push(AstItem::parse(lexer)?);
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

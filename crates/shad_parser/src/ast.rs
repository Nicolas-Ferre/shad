use crate::token::{Lexer, Token};
use crate::AstItem;
use fxhash::FxHashMap;
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
    /// The next available unique ID.
    pub next_id: u64,
}

impl Ast {
    /// Parses all Shad files in a directory to obtain their ASTs.
    ///
    /// The returned [`FxHashMap`] contains dot-separated module path as keys,
    /// and parsed files as values.
    ///
    /// Shad files in subdirectories are also parsed recursively.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn from_dir(path: impl AsRef<Path>) -> Result<FxHashMap<String, Self>, Error> {
        let path = path.as_ref();
        let mut next_id = 0;
        Ok(fs::read_dir(path)
            .map_err(Error::Io)?
            .map(|entry| match entry {
                Ok(entry) => {
                    let file_path = entry.path();
                    if file_path.is_dir() {
                        Self::from_dir(file_path)
                    } else {
                        let module = Self::path_to_module(path, &file_path);
                        Self::parse_file(&file_path, &module, next_id).map(|ast| {
                            next_id = ast.next_id;
                            iter::once((module, ast)).collect()
                        })
                    }
                }
                Err(err) => Err(Error::Io(err)),
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// Parses a file containing Shad code to obtain an AST.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn from_file(path: impl AsRef<Path>, module_name: &str) -> Result<Self, Error> {
        Self::parse_file(path, module_name, 0)
    }

    fn parse_file(path: impl AsRef<Path>, module_name: &str, next_id: u64) -> Result<Self, Error> {
        let code = Self::retrieve_code(&path).map_err(Error::Io)?;
        let path = path.as_ref().to_str().unwrap_or_default();
        let cleaned_code = Self::remove_comments(&code);
        let mut lexer = Lexer::new(&cleaned_code, path, module_name, next_id);
        Self::parse_str(&mut lexer, &code, path)
            .map_err(|e| e.with_pretty_message(path, &code))
            .map_err(Error::Syntax)
    }

    fn parse_str(lexer: &mut Lexer<'_>, code: &str, path: &str) -> Result<Self, SyntaxError> {
        let mut items = vec![];
        while Token::next(&mut lexer.clone()).is_ok() {
            items.push(AstItem::parse(lexer)?);
        }
        let next_id = lexer.next_id();
        Ok(Self {
            code: code.to_string(),
            path: path.to_string(),
            items,
            next_id,
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

    fn path_to_module(base_path: &Path, path: &Path) -> String {
        path.iter()
            .skip(base_path.components().count())
            .map(|component| component.to_str().unwrap_or("<invalid>"))
            .collect::<Vec<_>>()
            .join(".")
    }
}

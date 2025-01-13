use crate::token::Lexer;
use crate::AstItem;
use fxhash::FxHashMap;
use shad_error::{Error, SyntaxError};
use std::ffi::OsStr;
use std::path::Path;
use std::{fs, io, iter};

const ALLOWED_FILE_EXTENSION: &str = "shd";

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
        Self::parse_dir(path, path)
    }

    /// Parses a file containing Shad code to obtain an AST.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub fn from_file(path: impl AsRef<Path>, module_name: &str) -> Result<Self, Error> {
        Self::parse_file(path, module_name)
    }

    fn parse_dir(path: &Path, base_path: &Path) -> Result<FxHashMap<String, Self>, Error> {
        Ok(fs::read_dir(path)
            .map_err(Error::Io)?
            .map(|entry| match entry {
                Ok(entry) => {
                    let file_path = entry.path();
                    if file_path.is_dir() {
                        Self::parse_dir(&file_path, base_path)
                    } else if file_path.extension() == Some(OsStr::new(ALLOWED_FILE_EXTENSION)) {
                        let module = Self::path_to_module(base_path, &file_path);
                        Self::parse_file(&file_path, &module)
                            .map(|ast| iter::once((module, ast)).collect())
                    } else {
                        Ok(FxHashMap::default())
                    }
                }
                Err(err) => Err(Error::Io(err)), // no-coverage (difficult to test)
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    fn parse_file(path: impl AsRef<Path>, module_name: &str) -> Result<Self, Error> {
        let path = path.as_ref().to_str().unwrap_or_default();
        let raw_code = Self::retrieve_code(&path).map_err(Error::Io)?;
        let cleaned_code = Self::remove_comments(&raw_code);
        let mut lexer = Lexer::new(&cleaned_code, &raw_code, path, module_name);
        Self::parse_str(&mut lexer, &raw_code, path).map_err(Error::Syntax)
    }

    fn parse_str(lexer: &mut Lexer<'_>, code: &str, path: &str) -> Result<Self, SyntaxError> {
        let mut items = vec![];
        while lexer.has_next_token() {
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

    fn path_to_module(base_path: &Path, path: &Path) -> String {
        let segment_count = path.components().count() - base_path.components().count();
        path.iter()
            .skip(base_path.components().count())
            .take(segment_count - 1)
            .chain(path.file_stem())
            .map(|component| component.to_str().unwrap_or("<invalid>"))
            .collect::<Vec<_>>()
            .join(".")
    }
}

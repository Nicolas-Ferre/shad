pub(crate) mod ast;
pub(crate) mod error;
pub(crate) mod index;
pub(crate) mod parsing;
pub(crate) mod reading;
pub(crate) mod transpilation;
pub(crate) mod validation;

use crate::compilation::ast::AstNode;
use crate::compilation::index::AstNodeIndex;
use crate::compilation::parsing::parse_files;
use crate::compilation::reading::SourceFolder;
use crate::compilation::transpilation::transpile_asts;
use crate::compilation::validation::validate_asts;
use crate::{config, Program};
use error::Error;
use std::collections::HashMap;
use std::rc::Rc;

const FILE_EXT: &str = "shd";

/// Compiles Shad files in a given folder.
///
/// # Errors
///
/// An error is returned if the files cannot be compiled.
pub fn compile(folder: impl SourceFolder) -> Result<Program, Error> {
    let config = config::load_config().expect("internal error: config should be valid");
    let root_path = folder.path();
    let files = reading::read_files(folder).map_err(Error::Io)?;
    let mut asts = parse_files(&config, &files)?
        .into_iter()
        .map(|(path, (code, ast))| {
            (
                path,
                FileAst {
                    code,
                    index: AstNodeIndex::new(&ast),
                    root: ast,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    AstNodeIndex::generate_lookup_paths(&config, &mut asts, &root_path);
    validate_asts(&asts, &root_path)?;
    Ok(transpile_asts(&config, &asts, &root_path))
}

#[derive(Debug)]
pub(crate) struct FileAst {
    pub(crate) code: String,
    pub(crate) index: AstNodeIndex,
    pub(crate) root: Rc<AstNode>,
}

use crate::compilation::index::NodeIndex;
use crate::compilation::parsing::{parse_file, parse_files};
use crate::{Error, Program, SourceFolder};
use std::collections::HashMap;
use std::path::Path;

pub(crate) mod error;
pub(crate) mod index;
pub(crate) mod node;
pub(crate) mod parsing;
pub(crate) mod reading;
pub(crate) mod transpilation;
pub(crate) mod validation;

pub(crate) const FILE_EXT: &str = "shd";
const PRELUDE_PATH: &str = "prelude.shd";
const PRELUDE_CODE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/prelude.shd"
));

/// Compiles Shad files in a given folder.
///
/// # Errors
///
/// An error is returned if the files cannot be compiled.
pub fn compile(folder: impl SourceFolder) -> Result<Program, Error> {
    let root_path = folder.path();
    let files = reading::read_files(folder).map_err(Error::Io)?;
    let (prelude_root, next_node_id) = parse_file(Path::new(PRELUDE_PATH), PRELUDE_CODE, 0)
        .map_err(|err| Error::Parsing(vec![err]))?;
    let roots = parse_files(&files, next_node_id)?
        .into_iter()
        .chain([(PRELUDE_PATH.into(), prelude_root)])
        .collect::<HashMap<_, _>>();
    let index = NodeIndex::new(&roots, &root_path);
    validation::run(&roots, &index, &root_path)?;
    let program = Program::new(&roots, &index, &root_path);
    Ok(program)
}

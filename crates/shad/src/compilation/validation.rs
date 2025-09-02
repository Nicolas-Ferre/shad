use crate::compilation::index::NodeIndex;
use crate::compilation::node::Node;
use crate::language::nodes::items::Root;
use crate::{Error, ValidationError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) struct ValidationContext<'a> {
    pub(crate) roots: &'a HashMap<PathBuf, Root>,
    pub(crate) index: &'a NodeIndex,
    pub(crate) root_path: &'a Path,
    pub(crate) errors: Vec<ValidationError>,
}

pub(crate) fn run(
    roots: &HashMap<PathBuf, Root>,
    index: &NodeIndex,
    root_path: &Path,
) -> Result<(), Error> {
    let errors = roots
        .values()
        .flat_map(|root| {
            let mut ctx = ValidationContext {
                roots,
                index,
                root_path,
                errors: vec![],
            };
            root.validate_nested(&mut ctx);
            ctx.errors
        })
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(errors))
    }
}

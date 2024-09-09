use crate::snippet_path;
use shad_runner::{Error, Runner};

#[test]
fn run_missing_file() {
    matches!(Runner::new(snippet_path("missing.shd")), Err(Error::Io(_)));
}

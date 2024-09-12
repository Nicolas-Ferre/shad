use crate::snippet_path;
use shad_error::Error;
use shad_runner::Runner;

#[test]
fn run_missing_file() {
    matches!(Runner::new(snippet_path("missing.shd")), Err(Error::Io(_)));
}

#[test]
fn access_invalid_buffer() {
    let runner = Runner::new(snippet_path("buffer_valid.shd")).unwrap();
    assert!(runner.buffer("invalid_name").is_empty());
}

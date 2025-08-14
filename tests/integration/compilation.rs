use shad::Error;
use std::path::Path;

#[test]
fn run_missing_folder() {
    matches!(
        shad::compile(Path::new("./cases_valid/missing")),
        Err(Error::Io(_))
    );
}

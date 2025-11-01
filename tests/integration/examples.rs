use std::path::{Path, PathBuf};

#[rstest::rstest]
fn run_valid_code(
    #[dirs]
    #[files("../examples/*")]
    path: PathBuf,
) {
    assert!(shad::compile(Path::new(&path)).is_ok());
}

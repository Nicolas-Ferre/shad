use std::path::{Path, PathBuf};

#[rstest::rstest]
fn compile_example(
    #[dirs]
    #[files("../examples/*")]
    path: PathBuf,
) {
    assert!(shad::compile(Path::new(&path)).is_ok());
}

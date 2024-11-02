use shad_runner::Runner;
use std::fs;
use std::path::PathBuf;

#[rstest::rstest]
fn run_invalid_code(#[files("./cases_invalid/code/*.shd")] path: PathBuf) {
    let path = PathBuf::from(format!(
        "./cases_invalid/code/{}",
        path.file_name().unwrap().to_str().unwrap()
    ));
    let result = Runner::new(&path);
    let actual = String::from_utf8(strip_ansi_escapes::strip(format!(
        "{}",
        result.expect_err("invalid code has successfully compiled")
    )))
    .unwrap();
    let case_name = path.file_stem().unwrap();
    let error_path = path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("expected/")
        .join(case_name);
    if error_path.exists() {
        assert_eq!(
            fs::read_to_string(error_path).unwrap(),
            actual,
            "mismatching result for invalid {case_name:?} case",
        );
    } else {
        fs::write(error_path, actual).unwrap();
        panic!("expected error saved on disk, please check and rerun the tests");
    }
}

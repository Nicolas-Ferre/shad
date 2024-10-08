use shad_runner::Runner;
use std::fs;

#[test]
fn run_invalid_code() {
    let mut should_rerun = false;
    for entry in fs::read_dir("./cases_invalid/code").unwrap() {
        let code_path = entry.unwrap().path();
        let result = Runner::new(&code_path);
        let expected = String::from_utf8(strip_ansi_escapes::strip(format!(
            "{}",
            result.expect_err("invalid code has successfully compiled")
        )))
        .unwrap();
        let case_name = code_path.file_stem().unwrap();
        let error_path = code_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("expected/")
            .join(case_name);
        if error_path.exists() {
            assert_eq!(
                fs::read_to_string(error_path).unwrap(),
                expected,
                "mismatching result for invalid {case_name:?} case",
            );
        } else {
            fs::write(error_path, expected).unwrap();
            should_rerun = true;
        }
    }
    assert!(
        !should_rerun,
        "expected error saved on disk, please check and rerun the tests"
    );
}

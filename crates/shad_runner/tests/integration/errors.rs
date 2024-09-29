use shad_error::Error;
use shad_runner::Runner;
use std::fs;

#[test]
fn run_invalid_code() {
    let mut should_rerun = false;
    for entry in fs::read_dir("./tests/error_cases/code").unwrap() {
        let code_path = entry.unwrap().path();
        let result = Runner::new(&code_path);
        let errors = String::from_utf8(strip_ansi_escapes::strip(
            match result.expect_err("invalid code has successfully compiled") {
                Error::Syntax(err) => format!("{err}"),
                Error::Semantic(err) => err
                    .iter()
                    .map(|err| format!("{err}"))
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                Error::Io(err) => format!("{err}"),
            },
        ))
        .unwrap();
        let error_path = code_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("error/")
            .join(code_path.file_stem().unwrap());
        if error_path.exists() {
            assert_eq!(fs::read_to_string(error_path).unwrap(), errors);
        } else {
            fs::write(error_path, errors).unwrap();
            should_rerun = true;
        }
    }
    assert!(
        !should_rerun,
        "expected error saved on disk, please check and rerun the tests"
    );
}

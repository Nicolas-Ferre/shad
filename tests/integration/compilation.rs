use std::path::Path;

#[test]
fn run_missing_folder() {
    assert_eq!(
        shad::compile(Path::new("./cases_valid/missing"))
            .expect_err("invalid code has successfully compiled")
            .render(),
        "\u{1b}[1m\u{1b}[91merror\u{1b}[0m\u{1b}[1m: ./cases_valid/missing: No such file or directory (os error 2)\u{1b}[0m"
    );
}

use shad_parser::{Error, Program, SyntaxError};
use std::io;
use std::io::ErrorKind;

#[test]
fn generate_syntax_error() {
    let formatted_error = "\u{1b}[1m\u{1b}[91merror\u{1b}[0m: \u{1b}[1mexpected `=`\u{1b}[0m\n \u{1b}[1m\u{1b}[94m-->\u{1b}[0m file:1:6\n\u{1b}[1m\u{1b}[94m  |\u{1b}[0m\n\u{1b}[1m\u{1b}[94m1 |\u{1b}[0m buf a;\n\u{1b}[1m\u{1b}[94m  |\u{1b}[0m\u{1b}[1m\u{1b}[91m      ^\u{1b}[0m \u{1b}[1m\u{1b}[91mhere\u{1b}[0m\n\u{1b}[1m\u{1b}[94m  |\u{1b}[0m";
    let parsed = Program::parse_str("buf a;", "file");
    let Err(err) = parsed else {
        panic!("incorrect error")
    };
    assert_eq!(
        err,
        Error::Syntax(SyntaxError {
            offset: 5,
            message: "expected `=`".into(),
            pretty_message: formatted_error.into(),
        })
    );
    assert_ne!(err, Error::Io(io::Error::new(ErrorKind::NotFound, "error")));
    assert_eq!(format!("{err}"), formatted_error);
}

#[test]
fn generate_io_error() {
    let formatted_error = "No such file or directory (os error 2)";
    let parsed = Program::parse_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/missing.shd"
    ));
    let Err(err) = parsed else {
        panic!("incorrect error")
    };
    assert_eq!(
        err,
        Error::Io(io::Error::new(ErrorKind::NotFound, formatted_error))
    );
    assert_eq!(format!("{err}"), formatted_error);
}

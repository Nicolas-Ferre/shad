use crate::{assert_semantic_error, assert_syntax_error, f32_buffer, snippet_path};
use shad_analyzer::{ErrorLevel, LocatedMessage};
use shad_parser::Span;
use shad_runner::Runner;

#[test]
fn run_valid() {
    let runner = Runner::new(snippet_path("buffer_valid.shd")).unwrap();
    runner.run();
    assert_eq!(f32_buffer(&runner, "buffer_name"), 42.);
}

#[test]
fn run_invalid_syntax() {
    let result = Runner::new(snippet_path("buffer_invalid_syntax.shd"));
    assert_syntax_error(&result, "expected item", 0);
}

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("buffer_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &["buffer with name `duplicated` is defined multiple times"],
        &[&vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: Span { start: 59, end: 69 },
                text: "duplicated buffer name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: Span { start: 4, end: 14 },
                text: "buffer with same name is defined here".into(),
            },
        ]],
    );
}

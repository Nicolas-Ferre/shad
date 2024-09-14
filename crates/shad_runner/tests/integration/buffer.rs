use crate::{assert_semantic_error, assert_syntax_error, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

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
                span: Span::new(59, 69),
                text: "duplicated buffer name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: Span::new(4, 14),
                text: "buffer with same name is defined here".into(),
            },
        ]],
    );
}

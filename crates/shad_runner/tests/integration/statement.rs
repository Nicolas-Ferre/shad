use crate::{assert_semantic_error, f32_buffer, i32_buffer, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
#[allow(clippy::decimal_literal_representation)]
fn run_valid() {
    let mut runner = Runner::new(snippet_path("statement_valid.shd")).unwrap();
    runner.run_step();
    assert_eq!(f32_buffer(&runner, "value1"), 2.);
    assert_eq!(i32_buffer(&runner, "value2"), 3);
    assert_eq!(i32_buffer(&runner, "value3"), 3);
}

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("statement_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &["expression assigned to `value1` has invalid type"],
        &[&vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(53, 59),
                text: "expression of type `i32`".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: Span::new(44, 50),
                text: "expected type `f32`".into(),
            },
        ]],
    );
}

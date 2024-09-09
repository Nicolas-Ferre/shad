use crate::{assert_semantic_error, assert_syntax_error, f32_buffer, snippet_path};
use shad_analyzer::{ErrorLevel, LocatedMessage};
use shad_parser::Span;
use shad_runner::Runner;

#[test]
fn run_valid() {
    let runner = Runner::new(snippet_path("expr_valid.shd")).unwrap();
    runner.run();
    assert_eq!(f32_buffer(&runner, "float_zero"), 0.);
    assert_eq!(f32_buffer(&runner, "float_no_frac_part"), 0.);
    assert_eq!(f32_buffer(&runner, "float_many_digits"), 123_456_700.);
    assert_eq!(f32_buffer(&runner, "float_max_int_digits"), 1.234_567_8e37);
    assert_eq!(f32_buffer(&runner, "float_underscores"), 123_456_700.);
}

#[test]
fn run_invalid_float_literal_syntax() {
    let result = Runner::new(snippet_path("expr_invalid_float_underscore_frac_part.shd"));
    assert_syntax_error(&result, "expected `;`", 16);
    let result = Runner::new(snippet_path("expr_invalid_float_underscore_int_part.shd"));
    assert_syntax_error(&result, "expected expression", 12);
}

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("expr_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &["float literal with too many digits in integer part"],
        &[&vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: Span { start: 28, end: 67 },
                text: "found 39 digits".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: Span { start: 28, end: 67 },
                text: "maximum 38 digits are expected".into(),
            },
        ]],
    );
}

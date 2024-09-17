use crate::{assert_semantic_error, assert_syntax_error, f32_buffer, i32_buffer, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
#[allow(clippy::decimal_literal_representation)]
fn run_valid() {
    let mut runner = Runner::new(snippet_path("statement_valid.shd")).unwrap();
    runner.run_step();
    assert_eq!(f32_buffer(&runner, "set_f32"), 2.);
    assert_eq!(i32_buffer(&runner, "set_i32"), 3);
    assert_eq!(i32_buffer(&runner, "set_from_local_var"), 3);
    assert_eq!(f32_buffer(&runner, "aliased_value"), 2.);
}

#[test]
fn run_invalid_syntax() {
    let result = Runner::new(snippet_path("statement_invalid_syntax.shd"));
    assert_syntax_error(&result, "expected statement", 10);
}

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("statement_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &[
            "expression assigned to `f32_val` has invalid type",
            "expression assigned to `local_f32_val` has invalid type",
        ],
        &[
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(56, 63),
                    text: "expression of type `i32`".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(46, 53),
                    text: "expected type `f32`".into(),
                },
            ],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(113, 120),
                    text: "expression of type `i32`".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(97, 110),
                    text: "expected type `f32`".into(),
                },
            ],
        ],
    );
}

use crate::{assert_semantic_error, assert_syntax_error, f32_buffer, snippet_path, u32_buffer};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
#[allow(clippy::decimal_literal_representation)]
fn run_valid() {
    let runner = Runner::new(snippet_path("expr_valid.shd")).unwrap();
    runner.run();
    assert_eq!(f32_buffer(&runner, "f32_zero"), 0.);
    assert_eq!(f32_buffer(&runner, "f32_no_frac_part"), 0.);
    assert_eq!(f32_buffer(&runner, "f32_many_digits"), 123_456_700.);
    assert_eq!(f32_buffer(&runner, "f32_max_int_digits"), 1.234_567_8e37);
    assert_eq!(f32_buffer(&runner, "f32_underscores"), 123_456_700.);
    assert_eq!(u32_buffer(&runner, "u32_zero"), 0);
    assert_eq!(u32_buffer(&runner, "u32_underscores"), 123_456_789);
    assert_eq!(u32_buffer(&runner, "u32_max_value"), 4_294_967_295);
    assert_eq!(u32_buffer(&runner, "i32_zero"), 0);
    assert_eq!(u32_buffer(&runner, "i32_underscores"), 123_456_789);
    assert_eq!(u32_buffer(&runner, "i32_max_value"), 2_147_483_647);
    assert_eq!(u32_buffer(&runner, "copied_buffer"), 2_147_483_647);
}

#[test]
fn run_invalid_syntax() {
    let result = Runner::new(snippet_path("expr_invalid_empty.shd"));
    assert_syntax_error(&result, "expected expression", 12);
    let result = Runner::new(snippet_path("expr_invalid_underscore_f32_frac_part.shd"));
    assert_syntax_error(&result, "expected `;`", 16);
    let result = Runner::new(snippet_path("expr_invalid_underscore_f32_int_part.shd"));
    assert_syntax_error(&result, "unexpected token", 16);
}

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("expr_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &[
            "could not find `undefined` value",
            "could not find `f32_too_many_digits` value",
            "`f32` literal with too many digits in integer part",
            "`u32` literal out of range",
            "`i32` literal out of range",
        ],
        &[
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(22, 31),
                text: "undefined identifier".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(55, 74),
                text: "undefined identifier".into(),
            }],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(102, 141),
                    text: "found 39 digits".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(102, 141),
                    text: "maximum 38 digits are expected".into(),
                },
            ],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(162, 176),
                text: "value is outside allowed range for `u32` type".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(196, 209),
                text: "value is outside allowed range for `i32` type".into(),
            }],
        ],
    );
}

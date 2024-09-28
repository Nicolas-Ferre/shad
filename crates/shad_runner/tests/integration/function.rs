use crate::{assert_semantic_error, f32_buffer, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
#[allow(clippy::decimal_literal_representation, clippy::cognitive_complexity)]
fn run_valid() {
    let mut runner = Runner::new(snippet_path("fn_valid.shd")).unwrap();
    runner.run_step();
    assert_eq!(f32_buffer(&runner, "buffer"), 24.);
}

#[test]
#[allow(clippy::too_many_lines)]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("fn_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &[
            "parameter `param` is defined multiple times",
            "function `duplicated_fn(u32, u32)` is defined multiple times",
            "could not find `undef` type",
            "function `__add__` has an invalid number of parameters",
            "function `__neg__` has an invalid number of parameters",
            "`buf` function `buffer_fn()` called in invalid context",
            "invalid type for returned expression",
            "statement found after `return` statement",
            "`buf` function `buffer_fn()` called in invalid context",
            "could not find `buffer` value",
            "`return` statement used outside function",
        ],
        &[
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(36, 41),
                    text: "duplicated parameter".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(24, 29),
                    text: "parameter with same name is defined here".into(),
                },
            ],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(118, 131),
                    text: "duplicated function".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(63, 76),
                    text: "function with same signature is defined here".into(),
                },
            ],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(195, 200),
                text: "undefined type".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(217, 224),
                text: "found 1 parameters, expected 2".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(252, 259),
                text: "found 2 parameters, expected 1".into(),
            }],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(375, 386),
                    text: "this function cannot be called here".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(375, 386),
                    text:
                        "`buf` functions can only be called in `run` blocks and `buf fn` functions"
                            .into(),
                },
            ],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(533, 535),
                    text: "expression of type `f32`".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span {
                        start: 516,
                        end: 519,
                    },
                    text: "expected type `i32`".into(),
                },
            ],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(595, 606),
                    text: "this statement cannot be defined after a `return` statement".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(581, 590),
                    text: "`return` statement defined here".into(),
                },
            ],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(472, 483),
                    text: "this function cannot be called here".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(472, 483),
                    text:
                        "`buf` functions can only be called in `run` blocks and `buf fn` functions"
                            .into(),
                },
            ],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(345, 351),
                text: "undefined identifier".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(636, 645),
                text: "invalid statement".into(),
            }],
        ],
    );
}

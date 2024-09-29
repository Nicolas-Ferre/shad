use crate::{assert_semantic_error, f32_buffer, i32_buffer, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
#[allow(clippy::decimal_literal_representation, clippy::cognitive_complexity)]
fn run_valid() {
    let mut runner = Runner::new(snippet_path("fn_valid.shd")).unwrap();
    runner.run_step();
    assert_eq!(f32_buffer(&runner, "result_from_fn"), 24.);
    assert_eq!(f32_buffer(&runner, "operator_result"), 13.);
    assert_eq!(i32_buffer(&runner, "no_return_value_result"), 1);
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
            "use of `return` in a function with no return type",
            "statement found after `return` statement",
            "`buf` function `buffer_fn()` called in invalid context",
            "expression assigned to `param` has invalid type",
            "could not find `buffer` value",
            "`return` statement used outside function",
            "function `with_return_type()` called as a statement while having a return type",
            "function `without_return_type()` in an expression while not having a return type",
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
                    span: Span::new(543, 548),
                    text: "expression of type `f32`".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span {
                        start: 526,
                        end: 529,
                    },
                    text: "expected type `i32`".into(),
                },
            ],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(843, 852),
                text: "invalid statement".into(),
            }],
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(608, 619),
                    text: "this statement cannot be defined after a `return` statement".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(594, 603),
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
            &vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: Span::new(694, 695),
                    text: "expression of type `i32`".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: Span::new(686, 691),
                    text: "expected type `f32`".into(),
                },
            ],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(345, 351),
                text: "undefined identifier".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(724, 733),
                text: "invalid statement".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(866, 884),
                text: "returned value needs to be stored in a variable".into(),
            }],
            &vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: Span::new(898, 919),
                text: "this function cannot be called here".into(),
            }],
        ],
    );
}

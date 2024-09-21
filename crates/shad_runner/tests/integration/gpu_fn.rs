use crate::{assert_semantic_error, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("gpu_fn_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &[
            "parameter `param` is defined multiple times",
            "function with signature `duplicated_fn(u32, u32)` is defined multiple times",
            "could not find `undef` type",
            "function `__add__` has an invalid number of parameters",
            "function `__neg__` has an invalid number of parameters",
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
        ],
    );
}

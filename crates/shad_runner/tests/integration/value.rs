use crate::{assert_semantic_error, snippet_path};
use shad_error::{ErrorLevel, LocatedMessage, Span};
use shad_runner::Runner;

#[test]
fn run_invalid_semantic() {
    let result = Runner::new(snippet_path("value_invalid_semantic.shd"));
    assert_semantic_error(
        &result,
        &["could not find `undefined` value"],
        &[&vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: Span::new(28, 37),
            text: "undefined identifier".into(),
        }]],
    );
}

use crate::{f32_buffer, i32_buffer, snippet_path};
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

use crate::{f32_buffer, i32_buffer, snippet_path};
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

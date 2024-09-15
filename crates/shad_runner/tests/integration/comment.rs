use crate::{f32_buffer, snippet_path};
use shad_runner::Runner;

#[test]
fn run_valid() {
    let mut runner = Runner::new(snippet_path("comment_valid.shd")).unwrap();
    runner.run_step();
    assert_eq!(f32_buffer(&runner, "buffer_name"), 42.);
}

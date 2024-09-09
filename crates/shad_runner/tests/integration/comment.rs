use crate::{f32_buffer, snippet_path};
use shad_runner::Runner;

#[test]
fn run_valid() {
    let runner = Runner::new(snippet_path("comment_valid.shd")).unwrap();
    runner.run();
    assert_eq!(f32_buffer(&runner, "buffer_name"), 42.);
}

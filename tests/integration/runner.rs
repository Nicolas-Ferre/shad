use shad_analyzer::BufferId;
use shad_error::Error;
use shad_runner::Runner;
use std::time::{Duration, Instant};

#[test]
fn run_missing_file() {
    matches!(
        Runner::new("./cases_valid/code/missing/main.shd"),
        Err(Error::Io(_))
    );
}

#[test]
fn access_invalid_buffer() {
    let runner = Runner::new("./cases_valid/code/atom/main.shd").unwrap();
    assert!(runner
        .buffer(&BufferId {
            module: "main".into(),
            name: "invalid_name".into()
        })
        .is_empty());
}

#[test]
fn retrieve_delta() {
    let mut runner = Runner::new("./cases_valid/code/atom/main.shd").unwrap();
    let start = Instant::now();
    runner.run_step();
    let end = Instant::now();
    assert!(runner.delta() > Duration::ZERO);
    assert!(runner.delta() <= end - start);
}

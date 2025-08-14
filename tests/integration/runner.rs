use shad::Runner;
use std::path::Path;
use std::time::Instant;

#[test]
fn retrieve_delta_time() {
    let program = shad::compile(Path::new("./cases_valid/expressions")).unwrap();
    let start = Instant::now();
    let mut runner = Runner::new(program, None, Some((4, 4)));
    assert_eq!(runner.delta_secs(), 0.);
    runner.run_step();
    let end = Instant::now();
    assert!(runner.delta_secs() > 0.);
    assert!(runner.delta_secs() <= (end - start).as_secs_f32());
}

#[test]
fn access_non_existing_buffer() {
    let program = shad::compile(Path::new("./cases_valid/expressions")).unwrap();
    let runner = Runner::new(program, None, Some((4, 4)));
    assert!(runner.read("non_existing").is_empty());
}

#[test]
fn execute_init_shaders_only_once() {
    let buffer_name = "init.value";
    let program = shad::compile(Path::new("./cases_valid/blocks")).unwrap();
    let mut runner = Runner::new(program, None, Some((4, 4)));
    runner.run_step();
    assert_eq!(runner.read(buffer_name), &[2, 0, 0, 0]);
    runner.write(buffer_name, &[1, 0, 0, 0]);
    runner.run_step();
    assert_eq!(runner.read(buffer_name), &[1, 0, 0, 0]);
}

#[test]
fn execute_run_shaders_at_each_frame() {
    let buffer_name = "run.value1";
    let program = shad::compile(Path::new("./cases_valid/blocks")).unwrap();
    let mut runner = Runner::new(program, None, Some((4, 4)));
    runner.run_step();
    assert_eq!(runner.read(buffer_name), &[2, 0, 0, 0]);
    runner.write(buffer_name, &[1, 0, 0, 0]);
    runner.run_step();
    assert_eq!(runner.read(buffer_name), &[2, 0, 0, 0]);
}

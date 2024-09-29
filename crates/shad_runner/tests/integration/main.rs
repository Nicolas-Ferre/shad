#![allow(clippy::unwrap_used, clippy::float_cmp)]

use shad_runner::Runner;

mod comment;
mod errors;
mod expr;
mod function;
mod runner;
mod statement;

fn snippet_path(filename: &str) -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snippets/").to_string() + filename
}

fn f32_buffer(runner: &Runner, buffer_name: &str) -> f32 {
    let bytes = runner.buffer(buffer_name);
    assert_eq!(bytes.len(), 4);
    let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
    f32::from_ne_bytes(bytes)
}

fn u32_buffer(runner: &Runner, buffer_name: &str) -> u32 {
    let bytes = runner.buffer(buffer_name);
    assert_eq!(bytes.len(), 4);
    let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
    u32::from_ne_bytes(bytes)
}

fn i32_buffer(runner: &Runner, buffer_name: &str) -> i32 {
    let bytes = runner.buffer(buffer_name);
    assert_eq!(bytes.len(), 4);
    let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
    i32::from_ne_bytes(bytes)
}

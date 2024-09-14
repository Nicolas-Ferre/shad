#![allow(clippy::unwrap_used, clippy::float_cmp)]

use shad_error::{Error, LocatedMessage};
use shad_runner::Runner;

mod buffer;
mod comment;
mod expr;
mod gpu_fn;
mod runner;

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

fn assert_semantic_error(
    result: &Result<Runner, Error>,
    messages: &[&str],
    located_messages: &[&Vec<LocatedMessage>],
) {
    let error = result.as_ref().expect_err("invalid result");
    let Error::Semantic(errors) = error else {
        panic!("invalid error type");
    };
    let actual_messages: Vec<_> = errors.iter().map(|err| &err.message).collect();
    let actual_located_messages: Vec<_> = errors.iter().map(|err| &err.located_messages).collect();
    assert_eq!(actual_messages, messages);
    assert_eq!(actual_located_messages, located_messages);
}

fn assert_syntax_error(result: &Result<Runner, Error>, message: &str, offset: usize) {
    let error = result.as_ref().expect_err("invalid result");
    let Error::Syntax(error) = error else {
        panic!("invalid error type");
    };
    assert_eq!(error.message, message);
    assert_eq!(error.offset, offset);
}

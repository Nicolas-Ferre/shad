#![allow(
    missing_docs,
    clippy::print_stdout,
    clippy::use_debug,
    clippy::dbg_macro
)] // TODO: remove

use shad_parser::Program;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let parsed = Program::parse_file(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/complex.shd"));
    let end = Instant::now();
    match parsed {
        Ok(parsed) => println!("{parsed:#?}"),
        Err(err) => println!("{err}"),
    }
    println!("Parsing time: {}ms", (end - start).as_secs_f32() * 1000.);
}

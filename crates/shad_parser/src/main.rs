#![allow(
    missing_docs,
    clippy::print_stdout,
    clippy::use_debug,
    clippy::dbg_macro
)] // TODO: remove

use shad_parser::Program;
use std::time::Instant;

// TODO: improve performance
// TODO: improve error handling

fn main() {
    let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/complex.shd"));
    let start = Instant::now();
    let parsed = Program::parse(code);
    let end = Instant::now();
    match parsed {
        Ok(parsed) => {
            dbg!(parsed);
        }
        Err(err) => println!("{err}"),
    }
    println!("Parsing time: {}ms", (end - start).as_secs_f32() * 1000.);
}

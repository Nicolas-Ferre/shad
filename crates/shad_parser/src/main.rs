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
    let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/big.shd"));
    let start = Instant::now();
    let parsed = Program::parse(code);
    println!("Parsing time: {}ms", start.elapsed().as_secs_f32() * 1000.);
    match parsed {
        Ok(parsed) => {
            dbg!(parsed);
        }
        Err(err) => println!("{err}"),
    }
}

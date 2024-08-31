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
    let parsed = Program::parse(code);
    match parsed {
        Ok(parsed) => {
            dbg!(parsed);
        }
        // Ok(parsed) => println!("{parsed:?}"),
        Err(err) => println!("{err}"),
    }
}

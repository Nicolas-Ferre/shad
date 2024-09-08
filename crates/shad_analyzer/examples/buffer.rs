#![allow(missing_docs, clippy::print_stdout, clippy::use_debug)] // TODO: remove

use shad_analyzer::AnalyzedProgram;
use shad_parser::ParsedProgram;
use std::process;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let parsed =
        ParsedProgram::parse_file(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/buffer.shd"));
    let end = Instant::now();
    match parsed {
        Ok(parsed) => {
            // println!("{parsed:#?}");
            let analyzed = AnalyzedProgram::new(&parsed);
            if analyzed.errors().next().is_none() {
                println!("{analyzed:#?}");
            } else {
                for error in analyzed.errors() {
                    println!("{error}");
                }
            }
        }
        Err(err) => {
            println!("{err}");
            process::exit(1);
        }
    }
    println!("Parsing time: {}ms", (end - start).as_secs_f32() * 1000.);
}

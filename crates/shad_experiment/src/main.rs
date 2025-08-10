#![allow(missing_docs)]

use std::path::Path;

fn main() {
    match shad_experiment::compile(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/resources"))) {
        Ok(shaders) => {
            for shader in shaders {
                println!("{shader}");
                println!();
                println!("==============================================");
                println!();
            }
        }
        Err(err) => {
            println!("{}", err.render());
        }
    };
}

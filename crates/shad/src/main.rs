//! Shad language CLI.
#![allow(clippy::print_stdout, clippy::use_debug)]

use clap::Parser;
use itertools::Itertools;
use shad_analyzer::Asg;
use shad_parser::Ast;
use shad_runner::Runner;
use std::process;

// coverage: off (not simple to test)

fn main() {
    Args::parse().run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
enum Args {
    /// Run a Shad script
    Run(RunArgs),
    /// Display the Abstract Syntax Tree of a Shad script.
    Ast(AstArgs),
    /// Display the Abstract Semantic Graph of a Shad script.
    Asg(AsgArgs),
}

impl Args {
    fn run(self) {
        match self {
            Self::Run(args) => args.run(),
            Self::Ast(args) => args.run(),
            Self::Asg(args) => args.run(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct RunArgs {
    /// Path to the Shad script to run
    path: String,
    /// List of buffers to display at the end
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    buffer: Vec<String>,
    /// Number of steps to run (0 to run indefinitely)
    #[arg(short, long, default_value_t = 0)]
    steps: u32,
}

impl RunArgs {
    fn run(self) {
        match Runner::new(&self.path) {
            Ok(mut runner) => {
                if self.steps == 0 {
                    loop {
                        self.run_step(&mut runner);
                    }
                } else {
                    for _ in 0..self.steps {
                        self.run_step(&mut runner);
                    }
                }
            }
            Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        }
    }

    fn run_step(&self, runner: &mut Runner) {
        runner.run_step();
        for buffer in &self.buffer {
            println!("Buffer `{buffer}`: {:?}", runner.buffer(buffer));
        }
        println!("Step duration: {}µs", runner.delta().as_micros());
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct AstArgs {
    /// Path to the Shad script to run
    path: String,
}

impl AstArgs {
    fn run(self) {
        match Ast::from_file(self.path) {
            Ok(ast) => println!("{ast:#?}"),
            Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct AsgArgs {
    /// Path to the Shad script to run
    path: String,
}

impl AsgArgs {
    #[allow(clippy::similar_names)]
    fn run(self) {
        let ast = match Ast::from_file(self.path) {
            Ok(ast) => ast,
            Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        };
        let asg = Asg::analyze(&ast);
        if asg.errors.is_empty() {
            println!("{asg:#?}");
        } else {
            asg.errors
                .iter()
                .sorted_unstable_by_key(|err| err.located_messages[0].span.start)
                .for_each(|err| println!("{err}"));
            process::exit(1);
        }
    }
}

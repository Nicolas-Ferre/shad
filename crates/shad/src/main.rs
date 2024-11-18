//! Shad language CLI.
#![allow(clippy::print_stdout, clippy::use_debug)]

use clap::Parser;
use shad_analyzer::BufferId;
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
    /// Display the analysis result of a Shad script.
    Analyze(AnalyzeArgs),
}

impl Args {
    fn run(self) {
        match self {
            Self::Run(args) => args.run(),
            Self::Analyze(args) => args.run(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct RunArgs {
    /// Path to the Shad file or folder to run
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
            let buffer_id = buffer.rsplit_once('.').map_or_else(
                || BufferId {
                    module: String::new(),
                    name: buffer.into(),
                },
                |(module, name)| BufferId {
                    module: module.into(),
                    name: name.into(),
                },
            );
            println!("Buffer `{buffer}`: {:?}", runner.buffer(&buffer_id));
        }
        println!("Step duration: {}µs", runner.delta().as_micros());
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct AnalyzeArgs {
    /// Path to the Shad file or folder to analyze
    path: String,
}

impl AnalyzeArgs {
    #[allow(clippy::similar_names)]
    fn run(self) {
        match Runner::new(&self.path) {
            Ok(runner) => {
                println!("{:#?}", runner.analysis());
            }
            Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        }
    }
}
